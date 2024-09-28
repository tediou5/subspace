//! This module exposed implementation of farms maintenance.
//!
//! The goal is to observe farms in a cluster and keep controller's data structures
//! about which pieces are plotted in which sectors of which farm up to date. Implementation
//! automatically handles dynamic farm addition and removal, etc.

use crate::commands::cluster::farmer::FARMER_IDENTIFICATION_BROADCAST_INTERVAL;
use anyhow::anyhow;
use async_lock::RwLock as AsyncRwLock;
use futures::channel::oneshot;
use futures::future::FusedFuture;
use futures::stream::FuturesUnordered;
use futures::{select, FutureExt, Stream, StreamExt};
use parking_lot::Mutex;
use std::collections::hash_map::Entry;
use std::collections::{HashMap, HashSet, VecDeque};
use std::future::{pending, ready, Future};
use std::mem;
use std::pin::{pin, Pin};
use std::sync::Arc;
use std::time::Instant;
use subspace_core_primitives::{Blake3Hash, SectorIndex};
use subspace_farmer::cluster::controller::ClusterControllerFarmerIdentifyBroadcast;
use subspace_farmer::cluster::farmer::{
    ClusterFarm, ClusterFarmerFarmDetails, ClusterFarmerFarmDetailsRequest,
    ClusterFarmerIdentifyBroadcast, FarmIndex,
};
use subspace_farmer::cluster::nats_client::NatsClient;
use subspace_farmer::farm::plotted_pieces::PlottedPieces;
use subspace_farmer::farm::{Farm, FarmId, FarmerId, SectorPlottingDetails, SectorUpdate};
use tokio::task;
use tokio::time::MissedTickBehavior;
use tracing::{debug, error, info, trace, warn};

type AddRemoveFuture<'a> =
    Pin<Box<dyn Future<Output = Option<(FarmIndex, oneshot::Receiver<()>, ClusterFarm)>> + 'a>>;

#[derive(Debug)]
struct KnownFarmer {
    farmer_id: FarmerId,
    fingerprint: Blake3Hash,
    known_farms: HashMap<FarmIndex, KnownFarm>,
}

#[derive(Debug)]
struct KnownFarm {
    farm_id: FarmId,
    fingerprint: Blake3Hash,
    last_identification: Instant,
    expired_sender: oneshot::Sender<()>,
}

enum KnownFarmerInsertResult {
    Inserted,
    FingerprintUpdated {
        old_farms: HashMap<FarmId, (FarmIndex, KnownFarm)>,
    },
    NotInserted,
}

struct KnownFarmInsertResult {
    farm_index: FarmIndex,
    farm_details: ClusterFarmerFarmDetails,
    expired_receiver: oneshot::Receiver<()>,
    add: bool,
    remove: bool,
}

#[derive(Debug, Default)]
struct KnownFarms {
    known_farmers: Vec<KnownFarmer>,
    known_farms: HashMap<FarmIndex, KnownFarm>,
}

impl KnownFarms {
    async fn update_farmer<OP, Fut, S>(
        &mut self,
        farmer_id: FarmerId,
        fingerprint: Blake3Hash,
        farms_stream_op: OP,
    ) -> Vec<KnownFarmInsertResult>
    where
        OP: Fn() -> Fut,
        Fut: Future<Output = Option<S>>,
        S: Stream<Item = ClusterFarmerFarmDetails> + Unpin,
    {
        let last_identification = Instant::now();
        let result = self.known_farmers.iter_mut().find_map(|known_farmer| {
            let check_farmer_id = known_farmer.farmer_id == farmer_id;
            let check_fingerprint = known_farmer.fingerprint == fingerprint;
            match (check_farmer_id, check_fingerprint) {
                (true, true) => {
                    known_farmer
                        .known_farms
                        .iter_mut()
                        .for_each(|(_, known_farm)| {
                            debug!(%farmer_id, farm_id = %known_farm.farm_id, "Updating last identification for farm");
                            known_farm.last_identification = last_identification;
                        });
                    Some(KnownFarmerInsertResult::NotInserted)
                }
                (true, false) => {
                    let old_farms = known_farmer.known_farms.drain().map(|(farm_index, know_farm)| (know_farm.farm_id, (farm_index, know_farm))).collect();
                    known_farmer.fingerprint = fingerprint;
                    Some(KnownFarmerInsertResult::FingerprintUpdated{
                        old_farms,
                    })
                },
                (false, _) => None,
            }
        }).unwrap_or(KnownFarmerInsertResult::Inserted);

        if let KnownFarmerInsertResult::NotInserted = result {
            return vec![];
        }

        let Some(farms_stream) = farms_stream_op().await else {
            return vec![];
        };
        let farms = farms_stream.collect::<Vec<_>>().await;

        match result {
            KnownFarmerInsertResult::Inserted => {
                let mut known_farmer = KnownFarmer {
                    farmer_id,
                    fingerprint,
                    known_farms: HashMap::new(),
                };

                let res = farms
                    .into_iter()
                    .filter_map(|farm_details| {
                        let farm_index = self.pick_farmer_index()?;
                        let (expired_sender, expired_receiver) = oneshot::channel();
                        known_farmer.known_farms.insert(
                            farm_index,
                            KnownFarm {
                                farm_id: farm_details.farm_id,
                                fingerprint: farm_details.fingerprint,
                                last_identification,
                                expired_sender,
                            },
                        );
                        Some(KnownFarmInsertResult {
                            farm_index,
                            farm_details,
                            expired_receiver,
                            add: true,
                            remove: false,
                        })
                    })
                    .collect::<Vec<_>>();
                self.known_farmers.push(known_farmer);
                res
            }
            KnownFarmerInsertResult::FingerprintUpdated { mut old_farms } => {
                farms
                    .into_iter()
                    .filter_map(|farm_details| {
                        let farm_id = farm_details.farm_id;
                        let fingerprint = farm_details.fingerprint;
                        if let Some((farm_index, mut known_farm)) = old_farms.remove(&farm_id) {
                            known_farm.last_identification = last_identification;
                            if known_farm.farm_id == farm_id {
                                let known_farmer = self
                                    .get_known_farmer(farmer_id)
                                    .expect("Farmer should be available");
                                if known_farm.fingerprint == fingerprint {
                                    // Do nothing if farm is already known
                                    known_farmer.known_farms.insert(farm_index, known_farm);
                                    None
                                } else {
                                    // Update fingerprint
                                    let (expired_sender, expired_receiver) = oneshot::channel();
                                    known_farm.expired_sender = expired_sender;
                                    known_farmer.known_farms.insert(farm_index, known_farm);
                                    Some(KnownFarmInsertResult {
                                        farm_index,
                                        farm_details,
                                        expired_receiver,
                                        add: true,
                                        remove: true,
                                    })
                                }
                            } else {
                                None
                            }
                        } else {
                            // Add new farm
                            let (expired_sender, expired_receiver) = oneshot::channel();
                            let farm_index = self.pick_farmer_index()?;
                            self.get_known_farmer(farmer_id)
                                .expect("Farmer should be available")
                                .known_farms
                                .insert(
                                    farm_index,
                                    KnownFarm {
                                        farm_id,
                                        fingerprint,
                                        last_identification,
                                        expired_sender,
                                    },
                                );
                            Some(KnownFarmInsertResult {
                                farm_index,
                                farm_details,
                                expired_receiver,
                                add: true,
                                remove: false,
                            })
                        }
                    })
                    .collect::<Vec<_>>()
            }
            KnownFarmerInsertResult::NotInserted => {
                unreachable!("KnownFarmerInsertResult::NotInserted should be handled above")
            }
        }
    }

    fn get_known_farmer(&mut self, farmer_id: FarmerId) -> Option<&mut KnownFarmer> {
        self.known_farmers
            .iter_mut()
            .find(|known_farmer| known_farmer.farmer_id == farmer_id)
    }

    fn pick_farmer_index(&self) -> Option<u16> {
        let used_indices = self
            .known_farmers
            .iter()
            .flat_map(|known_farmer| known_farmer.known_farms.keys())
            .collect::<HashSet<_>>();

        for farm_index in FarmIndex::MIN..=FarmIndex::MAX {
            if !used_indices.contains(&farm_index) {
                return Some(farm_index);
            }
        }

        warn!(max_supported_farm_index = %FarmIndex::MAX, "Too many farms");
        None
    }

    fn insert_or_update(
        &mut self,
        farm_id: FarmId,
        fingerprint: Blake3Hash,
    ) -> KnownFarmInsertResult {
        // if let Some(existing_result) =
        //     self.known_farms
        //         .iter_mut()
        //         .find_map(|(&farm_index, known_farm)| {
        //             if known_farm.farm_id == farm_id {
        //                 if known_farm.fingerprint == fingerprint {
        //                     known_farm.last_identification = Instant::now();
        //                     Some(KnownFarmInsertResult::NotInserted)
        //                 } else {
        //                     let (expired_sender, expired_receiver) = oneshot::channel();

        //                     known_farm.fingerprint = fingerprint;
        //                     known_farm.expired_sender = expired_sender;

        //                     Some(KnownFarmInsertResult::FingerprintUpdated {
        //                         farm_index,
        //                         expired_receiver,
        //                     })
        //                 }
        //             } else {
        //                 None
        //             }
        //         })
        // {
        //     return existing_result;
        // }

        // for farm_index in FarmIndex::MIN..=FarmIndex::MAX {
        //     if let Entry::Vacant(entry) = self.known_farms.entry(farm_index) {
        //         let (expired_sender, expired_receiver) = oneshot::channel();

        //         entry.insert(KnownFarm {
        //             farm_id,
        //             fingerprint,
        //             last_identification: Instant::now(),
        //             expired_sender,
        //         });

        //         return KnownFarmInsertResult::Inserted {
        //             farm_index,
        //             expired_receiver,
        //         };
        //     }
        // }

        // warn!(%farm_id, max_supported_farm_index = %FarmIndex::MAX, "Too many farms, ignoring");
        // KnownFarmInsertResult::NotInserted
        todo!()
    }

    fn remove_expired(&mut self) -> impl Iterator<Item = (FarmIndex, KnownFarm)> + '_ {
        let elapsed = FARMER_IDENTIFICATION_BROADCAST_INTERVAL * 2;
        // self.known_farmers
        //     .retain(move |known_farmer| known_farmer.last_identification.elapsed() <= elapsed);

        self.known_farms.extract_if(move |_farm_index, known_farm| {
            known_farm.last_identification.elapsed() > elapsed
        })
    }

    fn remove(&mut self, farm_index: FarmIndex) {
        self.known_farms.remove(&farm_index);
    }
}

pub(super) async fn maintain_farms(
    instance: &str,
    nats_client: &NatsClient,
    plotted_pieces: &Arc<AsyncRwLock<PlottedPieces<FarmIndex>>>,
) -> anyhow::Result<()> {
    let mut known_farms = KnownFarms::default();

    // Futures that need to be processed sequentially in order to add/remove farms, if farm was
    // added, future will resolve with `Some`, `None` if removed
    let mut farms_to_add_remove = VecDeque::<AddRemoveFuture>::new();
    // Farm that is being added/removed right now (if any)
    let mut farm_add_remove_in_progress = (Box::pin(ready(None)) as AddRemoveFuture).fuse();
    // Initialize with pending future so it never ends
    let mut farms = FuturesUnordered::from_iter([
        Box::pin(pending()) as Pin<Box<dyn Future<Output = (FarmIndex, anyhow::Result<()>)>>>
    ]);

    let farmer_identify_subscription = pin!(nats_client
        .subscribe_to_broadcasts::<ClusterFarmerIdentifyBroadcast>(None, None)
        .await
        .map_err(|error| anyhow!("Failed to subscribe to farmer identify broadcast: {error}"))?);

    // Request farmer to identify themselves
    if let Err(error) = nats_client
        .broadcast(&ClusterControllerFarmerIdentifyBroadcast, instance)
        .await
    {
        warn!(%error, "Failed to send farmer identification broadcast");
    }

    let mut farmer_identify_subscription = farmer_identify_subscription.fuse();
    let mut farm_pruning_interval = tokio::time::interval_at(
        (Instant::now() + FARMER_IDENTIFICATION_BROADCAST_INTERVAL * 2).into(),
        FARMER_IDENTIFICATION_BROADCAST_INTERVAL * 2,
    );
    farm_pruning_interval.set_missed_tick_behavior(MissedTickBehavior::Delay);

    loop {
        if farm_add_remove_in_progress.is_terminated() {
            if let Some(fut) = farms_to_add_remove.pop_front() {
                farm_add_remove_in_progress = fut.fuse();
            }
        }

        select! {
            (farm_index, result) = farms.select_next_some() => {
                known_farms.remove(farm_index);
                farms_to_add_remove.push_back(Box::pin(async move {
                    let plotted_pieces = Arc::clone(plotted_pieces);

                    let delete_farm_fut = task::spawn_blocking(move || {
                        plotted_pieces.write_blocking().delete_farm(farm_index);
                    });
                    if let Err(error) = delete_farm_fut.await {
                        error!(
                            %farm_index,
                            %error,
                            "Failed to delete farm that exited"
                        );
                    }

                    None
                }));

                match result {
                    Ok(()) => {
                        info!(%farm_index, "Farm exited successfully");
                    }
                    Err(error) => {
                        error!(%farm_index, %error, "Farm exited with error");
                    }
                }
            }
            maybe_identify_message = farmer_identify_subscription.next() => {
                let Some(identify_message) = maybe_identify_message else {
                    return Err(anyhow!("Farmer identify stream ended"));
                };

                process_farmer_identify_message(
                    identify_message,
                    nats_client,
                    &mut known_farms,
                    &mut farms_to_add_remove,
                    plotted_pieces,
                ).await;
            }
            _ = farm_pruning_interval.tick().fuse() => {
                for (farm_index, removed_farm) in known_farms.remove_expired() {
                    let farm_id = removed_farm.farm_id;

                    if removed_farm.expired_sender.send(()).is_ok() {
                        warn!(
                            %farm_index,
                            %farm_id,
                            "Farm expired and removed"
                        );
                    } else {
                        warn!(
                            %farm_index,
                            %farm_id,
                            "Farm exited before expiration notification"
                        );
                    }

                    farms_to_add_remove.push_back(Box::pin(async move {
                        let plotted_pieces = Arc::clone(plotted_pieces);

                        let delete_farm_fut = task::spawn_blocking(move || {
                            plotted_pieces.write_blocking().delete_farm(farm_index);
                        });
                        if let Err(error) = delete_farm_fut.await {
                            error!(
                                %farm_index,
                                %farm_id,
                                %error,
                                "Failed to delete farm that expired"
                            );
                        }

                        None
                    }));
                }
            }
            result = farm_add_remove_in_progress => {
                if let Some((farm_index, expired_receiver, farm)) = result {
                    farms.push(Box::pin(async move {
                        select! {
                            result = farm.run().fuse() => {
                                (farm_index, result)
                            }
                            _ = expired_receiver.fuse() => {
                                // Nothing to do
                                (farm_index, Ok(()))
                            }
                        }
                    }));
                }
            }
        }
    }
}

async fn process_farmer_identify_message<'a>(
    identify_message: ClusterFarmerIdentifyBroadcast,
    nats_client: &'a NatsClient,
    known_farms: &mut KnownFarms,
    farms_to_add_remove: &mut VecDeque<AddRemoveFuture<'a>>,
    plotted_pieces: &'a Arc<AsyncRwLock<PlottedPieces<FarmIndex>>>,
) {
    let ClusterFarmerIdentifyBroadcast {
        farmer_id,
        fingerprint,
    } = identify_message;

    // if !known_farms.update_farmer(farmer_id, fingerprint) {
    //     // Farmer already known, nothing to do
    //     debug!(%farmer_id, "Farmer already known, nothing to do");
    //     return;
    // };

    match nats_client
        .stream_request(
            ClusterFarmerFarmDetailsRequest,
            Some(&farmer_id.to_string()),
        )
        .await
    {
        Ok(farms_details) => {
            // while let Some(farm_details) = farms_details.next().await {
            //     let farm_identify_message = ClusterFarmerIdentifyFarmBroadcast {
            //         farm_id: farm_details.farm_id,
            //         total_sectors_count: farm_details.total_sectors_count,
            //         fingerprint,
            //     };

            //     process_farm_identify_message(
            //         farm_identify_message,
            //         nats_client,
            //         known_farms,
            //         farms_to_add_remove,
            //         plotted_pieces,
            //     )
            // }
        }
        Err(error) => warn!(
            %error,
            %farmer_id,
            "Failed to request farmer farm details"
        ),
    }
}

// fn process_farm_identify_message<'a>(
//     identify_message: ClusterFarmerIdentifyFarmBroadcast,
//     nats_client: &'a NatsClient,
//     known_farms: &mut KnownFarms,
//     farms_to_add_remove: &mut VecDeque<AddRemoveFuture<'a>>,
//     plotted_pieces: &'a Arc<AsyncRwLock<PlottedPieces<FarmIndex>>>,
// ) {
//     let ClusterFarmerIdentifyFarmBroadcast {
//         farm_id,
//         total_sectors_count,
//         fingerprint,
//     } = identify_message;
//     let (farm_index, expired_receiver, add, remove) =
//         match known_farms.insert_or_update(farm_id, fingerprint) {
//             KnownFarmInsertResult::Inserted {
//                 farm_index,
//                 expired_receiver,
//             } => {
//                 info!(
//                     %farm_index,
//                     %farm_id,
//                     "Discovered new farm, initializing"
//                 );

//                 (farm_index, expired_receiver, true, false)
//             }
//             KnownFarmInsertResult::FingerprintUpdated {
//                 farm_index,
//                 expired_receiver,
//             } => {
//                 info!(
//                     %farm_index,
//                     %farm_id,
//                     "Farm fingerprint updated, re-initializing"
//                 );

//                 (farm_index, expired_receiver, true, true)
//             }
//             KnownFarmInsertResult::NotInserted => {
//                 trace!(
//                     %farm_id,
//                     "Received identification for already known farm"
//                 );
//                 // Nothing to do here
//                 return;
//             }
//         };

//     if remove {
//         farms_to_add_remove.push_back(Box::pin(async move {
//             let plotted_pieces = Arc::clone(plotted_pieces);

//             let delete_farm_fut = task::spawn_blocking(move || {
//                 plotted_pieces.write_blocking().delete_farm(farm_index);
//             });
//             if let Err(error) = delete_farm_fut.await {
//                 error!(
//                     %farm_index,
//                     %farm_id,
//                     %error,
//                     "Failed to delete farm that was replaced"
//                 );
//             }

//             None
//         }));
//     }

//     if add {
//         farms_to_add_remove.push_back(Box::pin(async move {
//             match initialize_farm(
//                 farm_index,
//                 farm_id,
//                 total_sectors_count,
//                 Arc::clone(plotted_pieces),
//                 nats_client,
//             )
//             .await
//             {
//                 Ok(farm) => {
//                     if remove {
//                         info!(
//                             %farm_index,
//                             %farm_id,
//                             "Farm re-initialized successfully"
//                         );
//                     } else {
//                         info!(
//                             %farm_index,
//                             %farm_id,
//                             "Farm initialized successfully"
//                         );
//                     }

//                     Some((farm_index, expired_receiver, farm))
//                 }
//                 Err(error) => {
//                     warn!(
//                         %error,
//                         "Failed to initialize farm {farm_id}"
//                     );
//                     None
//                 }
//             }
//         }));
//     }

async fn remove_farm<'a>(
    farm_id: FarmId,
    farm_index: FarmIndex,
    farms_to_add_remove: &mut VecDeque<AddRemoveFuture<'a>>,
    plotted_pieces: Arc<AsyncRwLock<PlottedPieces<FarmIndex>>>,
) {
    farms_to_add_remove.push_back(Box::pin(async move {
        let delete_farm_fut = task::spawn_blocking(move || {
            plotted_pieces.write_blocking().delete_farm(farm_index);
        });
        if let Err(error) = delete_farm_fut.await {
            error!(
                %farm_index,
                %farm_id,
                %error,
                "Failed to delete farm that was replaced",
            );
        }

        None
    }));
}

async fn add_farm<'a>(
    farm_id: FarmId,
    farm_index: FarmIndex,
    total_sectors_count: u16,
    remove: bool,
    expired_receiver: oneshot::Receiver<()>,
    nats_client: NatsClient,
    farms_to_add_remove: &mut VecDeque<AddRemoveFuture<'a>>,
    plotted_pieces: Arc<AsyncRwLock<PlottedPieces<FarmIndex>>>,
) {
    farms_to_add_remove.push_back(Box::pin(async move {
        match initialize_farm(
            farm_index,
            farm_id,
            total_sectors_count,
            plotted_pieces,
            &nats_client,
        )
        .await
        {
            Ok(farm) => {
                if remove {
                    info!(
                        %farm_index,
                        %farm_id,
                        "Farm re-initialized successfully"
                    );
                } else {
                    info!(
                        %farm_index,
                        %farm_id,
                        "Farm initialized successfully"
                    );
                }

                Some((farm_index, expired_receiver, farm))
            }
            Err(error) => {
                warn!(
                    %error,
                    "Failed to initialize farm {farm_id}"
                );
                None
            }
        }
    }));
}
// }

async fn initialize_farm(
    farm_index: FarmIndex,
    farm_id: FarmId,
    total_sectors_count: SectorIndex,
    plotted_pieces: Arc<AsyncRwLock<PlottedPieces<FarmIndex>>>,
    nats_client: &NatsClient,
) -> anyhow::Result<ClusterFarm> {
    let farm = ClusterFarm::new(farm_id, total_sectors_count, nats_client.clone())
        .await
        .map_err(|error| anyhow!("Failed instantiate cluster farm {farm_id}: {error}"))?;

    // Buffer sectors that are plotted while already plotted sectors are being iterated over
    let plotted_sectors_buffer = Arc::new(Mutex::new(Vec::new()));
    let sector_update_handler = farm.on_sector_update(Arc::new({
        let plotted_sectors_buffer = Arc::clone(&plotted_sectors_buffer);

        move |(_sector_index, sector_update)| {
            if let SectorUpdate::Plotting(SectorPlottingDetails::Finished {
                plotted_sector,
                old_plotted_sector,
                ..
            }) = sector_update
            {
                plotted_sectors_buffer
                    .lock()
                    .push((plotted_sector.clone(), old_plotted_sector.clone()));
            }
        }
    }));

    // Add plotted sectors of the farm to global plotted pieces
    let plotted_sectors = farm.plotted_sectors();
    let mut plotted_sectors = plotted_sectors
        .get()
        .await
        .map_err(|error| anyhow!("Failed to get plotted sectors for farm {farm_id}: {error}"))?;

    {
        let mut plotted_pieces = plotted_pieces.write().await;
        plotted_pieces.add_farm(farm_index, farm.piece_reader());

        while let Some(plotted_sector_result) = plotted_sectors.next().await {
            let plotted_sector = plotted_sector_result.map_err(|error| {
                anyhow!("Failed to get plotted sector for farm {farm_id}: {error}")
            })?;

            plotted_pieces.add_sector(farm_index, &plotted_sector);

            task::yield_now().await;
        }
    }

    // Add sectors that were plotted while above iteration was happening to plotted sectors
    // too
    drop(sector_update_handler);
    let plotted_sectors_buffer = mem::take(&mut *plotted_sectors_buffer.lock());
    let add_buffered_sectors_fut = task::spawn_blocking(move || {
        let mut plotted_pieces = plotted_pieces.write_blocking();

        for (plotted_sector, old_plotted_sector) in plotted_sectors_buffer {
            if let Some(old_plotted_sector) = old_plotted_sector {
                plotted_pieces.delete_sector(farm_index, &old_plotted_sector);
            }
            // Call delete first to avoid adding duplicates
            plotted_pieces.delete_sector(farm_index, &plotted_sector);
            plotted_pieces.add_sector(farm_index, &plotted_sector);
        }
    });

    add_buffered_sectors_fut
        .await
        .map_err(|error| anyhow!("Failed to add buffered sectors for farm {farm_id}: {error}"))?;

    Ok(farm)
}
