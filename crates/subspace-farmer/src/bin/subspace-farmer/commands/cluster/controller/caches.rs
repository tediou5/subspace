//! This module exposed implementation of caches maintenance.
//!
//! The goal is to observe caches in a particular cache group and keep controller's data structures
//! about which pieces are stored where up to date. Implementation automatically handles dynamic
//! cache addition and removal, tries to reduce number of reinitializations that result in potential
//! piece cache sync, etc.

use crate::commands::cluster::cache::CACHE_IDENTIFICATION_BROADCAST_INTERVAL;
use anyhow::anyhow;
use futures::channel::oneshot;
use futures::future::FusedFuture;
use futures::{select, FutureExt, Stream, StreamExt};
use parking_lot::Mutex;
use std::future::{ready, Future};
use std::pin::{pin, Pin};
use std::sync::Arc;
use std::time::{Duration, Instant};
use subspace_farmer::cluster::cache::{
    ClusterCacheDetailsRequest, ClusterCacheIdentifyBroadcast, ClusterCacheIndex,
    ClusterPieceCache, ClusterPieceCacheDetails,
};
use subspace_farmer::cluster::controller::ClusterControllerCacheIdentifyBroadcast;
use subspace_farmer::cluster::nats_client::NatsClient;
use subspace_farmer::farm::{CacheId, PieceCache};
use subspace_farmer::farmer_cache::FarmerCache;
use tokio::time::MissedTickBehavior;
use tracing::{info, warn};

const SCHEDULE_REINITIALIZATION_DELAY: Duration = Duration::from_secs(3);

#[derive(Debug)]
struct KnownCache {
    cache_id: CacheId,
    last_identification: Instant,
    piece_caches: Vec<Arc<ClusterPieceCache>>,
}

#[derive(Debug, Default)]
struct KnownCaches {
    known_caches: Vec<KnownCache>,
}

impl KnownCaches {
    fn get_all(&self) -> Vec<Arc<dyn PieceCache>> {
        self.known_caches
            .iter()
            .flat_map(|known_cache| {
                known_cache
                    .piece_caches
                    .iter()
                    .map(|piece_cache| Arc::clone(piece_cache) as Arc<_>)
            })
            .collect()
    }

    /// Return `true` if farmer cache reinitialization is required
    async fn update_cache<OP, Fut, S>(
        &mut self,
        cache_id: CacheId,
        scheduled_reinitialization_for: &mut Option<Instant>,
        nats_client: &NatsClient,
        piece_cache_stream_op: OP,
    ) where
        OP: Fn() -> Fut,
        Fut: Future<Output = Option<S>>,
        S: Stream<Item = ClusterPieceCacheDetails> + Unpin,
    {
        let last_identification = Instant::now();
        if self.known_caches.iter_mut().any(|known_cache| {
            if known_cache.cache_id == cache_id {
                known_cache.last_identification = last_identification;
                true
            } else {
                false
            }
        }) {
            return;
        }

        let Some(piece_caches_stream) = piece_cache_stream_op().await else {
            return;
        };
        let piece_caches = piece_caches_stream
            .map(
                |ClusterPieceCacheDetails {
                     piece_cache_id,
                     max_num_elements,
                 }| {
                    Arc::new(ClusterPieceCache::new(
                        piece_cache_id,
                        max_num_elements,
                        nats_client.clone(),
                    ))
                },
            )
            .collect()
            .await;

        info!(%cache_id, "New cache discovered, scheduling reinitialization");
        scheduled_reinitialization_for.replace(Instant::now() + SCHEDULE_REINITIALIZATION_DELAY);

        self.known_caches.push(KnownCache {
            cache_id,
            last_identification,
            piece_caches,
        });
    }

    fn remove_expired(&mut self) -> impl Iterator<Item = Arc<ClusterPieceCache>> + '_ {
        self.known_caches
            .extract_if(move |known_cache| {
                known_cache.last_identification.elapsed()
                    > CACHE_IDENTIFICATION_BROADCAST_INTERVAL * 2
            })
            .flat_map(|known_cache| known_cache.piece_caches)
    }
}

pub(super) async fn maintain_caches(
    cache_group: &str,
    nats_client: &NatsClient,
    farmer_cache: FarmerCache<ClusterCacheIndex>,
) -> anyhow::Result<()> {
    let mut known_caches = KnownCaches::default();

    let mut scheduled_reinitialization_for = None;
    let mut cache_reinitialization =
        (Box::pin(ready(())) as Pin<Box<dyn Future<Output = ()>>>).fuse();

    let cache_identify_subscription = pin!(nats_client
        .subscribe_to_broadcasts::<ClusterCacheIdentifyBroadcast>(None, None)
        .await
        .map_err(|error| anyhow!("Failed to subscribe to cache identify broadcast: {error}"))?);

    // Request cache to identify themselves
    if let Err(error) = nats_client
        .broadcast(&ClusterControllerCacheIdentifyBroadcast, cache_group)
        .await
    {
        warn!(%error, "Failed to send cache identification broadcast");
    }

    let mut cache_identify_subscription = cache_identify_subscription.fuse();
    let mut cache_pruning_interval = tokio::time::interval_at(
        (Instant::now() + CACHE_IDENTIFICATION_BROADCAST_INTERVAL * 2).into(),
        CACHE_IDENTIFICATION_BROADCAST_INTERVAL * 2,
    );
    cache_pruning_interval.set_missed_tick_behavior(MissedTickBehavior::Delay);

    loop {
        if cache_reinitialization.is_terminated()
            && let Some(time) = scheduled_reinitialization_for
            && time <= Instant::now()
        {
            scheduled_reinitialization_for.take();

            let new_piece_caches = known_caches.get_all();
            let new_cache_reinitialization = async {
                let (sync_finish_sender, sync_finish_receiver) = oneshot::channel::<()>();
                let sync_finish_sender = Mutex::new(Some(sync_finish_sender));

                let _handler_id = farmer_cache.on_sync_progress(Arc::new(move |&progress| {
                    if progress == 100.0 {
                        if let Some(sync_finish_sender) = sync_finish_sender.lock().take() {
                            // Result doesn't matter
                            let _ = sync_finish_sender.send(());
                        }
                    }
                }));

                farmer_cache
                    .replace_backing_caches(new_piece_caches, Vec::new())
                    .await;

                // Wait for piece cache sync to finish before potentially staring a new one, result
                // doesn't matter
                let _ = sync_finish_receiver.await;
            };

            cache_reinitialization =
                (Box::pin(new_cache_reinitialization) as Pin<Box<dyn Future<Output = ()>>>).fuse();
        }

        select! {
            maybe_identify_message = cache_identify_subscription.next() => {
                let Some(identify_message) = maybe_identify_message else {
                    return Err(anyhow!("Cache identify stream ended"));
                };

                let ClusterCacheIdentifyBroadcast { cache_id } = identify_message;

                known_caches.update_cache(
                    cache_id,
                    &mut scheduled_reinitialization_for,
                    nats_client,
                    || async {
                        nats_client
                            .stream_request(
                                ClusterCacheDetailsRequest,
                                Some(&cache_id.to_string()),
                            )
                            .await
                            .inspect_err(|error| warn!(
                                %error,
                                %cache_id,
                                "Failed to request farmer farm details"
                            ))
                            .ok()
                    },
                ).await
            }
            _ = cache_pruning_interval.tick().fuse() => {
                let mut reinit = false;
                for removed_cache in known_caches.remove_expired() {
                    reinit = true;

                    warn!(
                        cache_id = %removed_cache.id(),
                        "Cache expired and removed, scheduling reinitialization"
                    );
                }

                if reinit {
                    scheduled_reinitialization_for.replace(
                        Instant::now() + SCHEDULE_REINITIALIZATION_DELAY,
                    );
                }
            }
            _ = cache_reinitialization => {
                // Nothing left to do
            }
        }
    }
}
