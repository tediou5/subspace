
//! Autogenerated weights for pallet_messenger
//!
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 42.0.0
//! DATE: 2024-09-06, STEPS: `50`, REPEAT: `20`, LOW RANGE: `[]`, HIGH RANGE: `[]`
//! WORST CASE MAP SIZE: `1000000`
//! HOSTNAME: `Ubuntu-2404-noble-amd64-base`, CPU: `Intel(R) Core(TM) i7-6700 CPU @ 3.40GHz`
//! EXECUTION: , WASM-EXECUTION: Compiled, CHAIN: None, DB CACHE: 1024

// Executed Command:
// ./target/release/subspace-node
// benchmark
// pallet
// --runtime=./target/release/wbuild/subspace-runtime/subspace_runtime.compact.compressed.wasm
// --genesis-builder=runtime
// --steps=50
// --repeat=20
// --pallet=pallet_messenger
// --extrinsic=*
// --wasm-execution=compiled
// --heap-pages=4096
// --output=./weights/pallet-messenger.rs
// --template=./frame-weight-template.hbs

#![cfg_attr(rustfmt, rustfmt_skip)]
#![allow(unused_parens)]
#![allow(unused_imports)]

use frame_support::{traits::Get, weights::{Weight, constants::ParityDbWeight}};
use core::marker::PhantomData;

/// Weight functions needed for pallet_messenger.
pub trait WeightInfo {
	fn initiate_channel() -> Weight;
	fn close_channel() -> Weight;
	fn do_open_channel() -> Weight;
	fn do_close_channel() -> Weight;
	fn relay_message() -> Weight;
	fn relay_message_response() -> Weight;
}

/// Weights for pallet_messenger using the Substrate node and recommended hardware.
pub struct SubstrateWeight<T>(PhantomData<T>);
impl<T: frame_system::Config> WeightInfo for SubstrateWeight<T> {
	/// Storage: `Messenger::ChainAllowlist` (r:1 w:0)
	/// Proof: `Messenger::ChainAllowlist` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `Messenger::NextChannelId` (r:1 w:1)
	/// Proof: `Messenger::NextChannelId` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `System::Account` (r:1 w:1)
	/// Proof: `System::Account` (`max_values`: None, `max_size`: Some(128), added: 2603, mode: `MaxEncodedLen`)
	/// Storage: `Balances::Holds` (r:1 w:1)
	/// Proof: `Balances::Holds` (`max_values`: None, `max_size`: Some(5550), added: 8025, mode: `MaxEncodedLen`)
	/// Storage: `Messenger::CounterForOutbox` (r:1 w:1)
	/// Proof: `Messenger::CounterForOutbox` (`max_values`: Some(1), `max_size`: Some(4), added: 499, mode: `MaxEncodedLen`)
	/// Storage: `Messenger::Outbox` (r:1 w:1)
	/// Proof: `Messenger::Outbox` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `Messenger::MessageWeightTags` (r:1 w:1)
	/// Proof: `Messenger::MessageWeightTags` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `Messenger::Channels` (r:0 w:1)
	/// Proof: `Messenger::Channels` (`max_values`: None, `max_size`: None, mode: `Measured`)
	fn initiate_channel() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `62`
		//  Estimated: `9015`
		// Minimum execution time: 84_969_000 picoseconds.
		Weight::from_parts(85_882_000, 9015)
			.saturating_add(T::DbWeight::get().reads(7_u64))
			.saturating_add(T::DbWeight::get().writes(7_u64))
	}
	/// Storage: `Messenger::Channels` (r:1 w:1)
	/// Proof: `Messenger::Channels` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `Messenger::CounterForOutbox` (r:1 w:1)
	/// Proof: `Messenger::CounterForOutbox` (`max_values`: Some(1), `max_size`: Some(4), added: 499, mode: `MaxEncodedLen`)
	/// Storage: `Messenger::Outbox` (r:1 w:1)
	/// Proof: `Messenger::Outbox` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `Messenger::MessageWeightTags` (r:1 w:1)
	/// Proof: `Messenger::MessageWeightTags` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	fn close_channel() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `250`
		//  Estimated: `3715`
		// Minimum execution time: 34_897_000 picoseconds.
		Weight::from_parts(35_649_000, 3715)
			.saturating_add(T::DbWeight::get().reads(4_u64))
			.saturating_add(T::DbWeight::get().writes(4_u64))
	}
	/// Storage: `Messenger::Channels` (r:1 w:1)
	/// Proof: `Messenger::Channels` (`max_values`: None, `max_size`: None, mode: `Measured`)
	fn do_open_channel() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `227`
		//  Estimated: `3692`
		// Minimum execution time: 11_311_000 picoseconds.
		Weight::from_parts(11_706_000, 3692)
			.saturating_add(T::DbWeight::get().reads(1_u64))
			.saturating_add(T::DbWeight::get().writes(1_u64))
	}
	/// Storage: `Messenger::Channels` (r:1 w:1)
	/// Proof: `Messenger::Channels` (`max_values`: None, `max_size`: None, mode: `Measured`)
	fn do_close_channel() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `227`
		//  Estimated: `3692`
		// Minimum execution time: 11_114_000 picoseconds.
		Weight::from_parts(11_477_000, 3692)
			.saturating_add(T::DbWeight::get().reads(1_u64))
			.saturating_add(T::DbWeight::get().writes(1_u64))
	}
	/// Storage: `Messenger::Inbox` (r:1 w:1)
	/// Proof: `Messenger::Inbox` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `Messenger::Channels` (r:1 w:1)
	/// Proof: `Messenger::Channels` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `Messenger::InboxResponses` (r:1 w:1)
	/// Proof: `Messenger::InboxResponses` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `Messenger::CounterForInboxResponses` (r:1 w:1)
	/// Proof: `Messenger::CounterForInboxResponses` (`max_values`: Some(1), `max_size`: Some(4), added: 499, mode: `MaxEncodedLen`)
	/// Storage: `Messenger::MessageWeightTags` (r:1 w:1)
	/// Proof: `Messenger::MessageWeightTags` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `Messenger::UpdatedChannels` (r:1 w:1)
	/// Proof: `Messenger::UpdatedChannels` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	fn relay_message() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `346`
		//  Estimated: `3811`
		// Minimum execution time: 36_673_000 picoseconds.
		Weight::from_parts(37_070_000, 3811)
			.saturating_add(T::DbWeight::get().reads(6_u64))
			.saturating_add(T::DbWeight::get().writes(6_u64))
	}
	/// Storage: `Messenger::OutboxResponses` (r:1 w:1)
	/// Proof: `Messenger::OutboxResponses` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `Messenger::Channels` (r:1 w:1)
	/// Proof: `Messenger::Channels` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `Messenger::Outbox` (r:1 w:1)
	/// Proof: `Messenger::Outbox` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `Messenger::CounterForOutbox` (r:1 w:1)
	/// Proof: `Messenger::CounterForOutbox` (`max_values`: Some(1), `max_size`: Some(4), added: 499, mode: `MaxEncodedLen`)
	/// Storage: `Messenger::MessageWeightTags` (r:1 w:1)
	/// Proof: `Messenger::MessageWeightTags` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `Messenger::UpdatedChannels` (r:1 w:1)
	/// Proof: `Messenger::UpdatedChannels` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	fn relay_message_response() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `540`
		//  Estimated: `4005`
		// Minimum execution time: 35_566_000 picoseconds.
		Weight::from_parts(36_336_000, 4005)
			.saturating_add(T::DbWeight::get().reads(6_u64))
			.saturating_add(T::DbWeight::get().writes(6_u64))
	}
}

// For backwards compatibility and tests
impl WeightInfo for () {
	/// Storage: `Messenger::ChainAllowlist` (r:1 w:0)
	/// Proof: `Messenger::ChainAllowlist` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `Messenger::NextChannelId` (r:1 w:1)
	/// Proof: `Messenger::NextChannelId` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `System::Account` (r:1 w:1)
	/// Proof: `System::Account` (`max_values`: None, `max_size`: Some(128), added: 2603, mode: `MaxEncodedLen`)
	/// Storage: `Balances::Holds` (r:1 w:1)
	/// Proof: `Balances::Holds` (`max_values`: None, `max_size`: Some(5550), added: 8025, mode: `MaxEncodedLen`)
	/// Storage: `Messenger::CounterForOutbox` (r:1 w:1)
	/// Proof: `Messenger::CounterForOutbox` (`max_values`: Some(1), `max_size`: Some(4), added: 499, mode: `MaxEncodedLen`)
	/// Storage: `Messenger::Outbox` (r:1 w:1)
	/// Proof: `Messenger::Outbox` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `Messenger::MessageWeightTags` (r:1 w:1)
	/// Proof: `Messenger::MessageWeightTags` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `Messenger::Channels` (r:0 w:1)
	/// Proof: `Messenger::Channels` (`max_values`: None, `max_size`: None, mode: `Measured`)
	fn initiate_channel() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `62`
		//  Estimated: `9015`
		// Minimum execution time: 84_969_000 picoseconds.
		Weight::from_parts(85_882_000, 9015)
			.saturating_add(ParityDbWeight::get().reads(7_u64))
			.saturating_add(ParityDbWeight::get().writes(7_u64))
	}
	/// Storage: `Messenger::Channels` (r:1 w:1)
	/// Proof: `Messenger::Channels` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `Messenger::CounterForOutbox` (r:1 w:1)
	/// Proof: `Messenger::CounterForOutbox` (`max_values`: Some(1), `max_size`: Some(4), added: 499, mode: `MaxEncodedLen`)
	/// Storage: `Messenger::Outbox` (r:1 w:1)
	/// Proof: `Messenger::Outbox` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `Messenger::MessageWeightTags` (r:1 w:1)
	/// Proof: `Messenger::MessageWeightTags` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	fn close_channel() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `250`
		//  Estimated: `3715`
		// Minimum execution time: 34_897_000 picoseconds.
		Weight::from_parts(35_649_000, 3715)
			.saturating_add(ParityDbWeight::get().reads(4_u64))
			.saturating_add(ParityDbWeight::get().writes(4_u64))
	}
	/// Storage: `Messenger::Channels` (r:1 w:1)
	/// Proof: `Messenger::Channels` (`max_values`: None, `max_size`: None, mode: `Measured`)
	fn do_open_channel() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `227`
		//  Estimated: `3692`
		// Minimum execution time: 11_311_000 picoseconds.
		Weight::from_parts(11_706_000, 3692)
			.saturating_add(ParityDbWeight::get().reads(1_u64))
			.saturating_add(ParityDbWeight::get().writes(1_u64))
	}
	/// Storage: `Messenger::Channels` (r:1 w:1)
	/// Proof: `Messenger::Channels` (`max_values`: None, `max_size`: None, mode: `Measured`)
	fn do_close_channel() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `227`
		//  Estimated: `3692`
		// Minimum execution time: 11_114_000 picoseconds.
		Weight::from_parts(11_477_000, 3692)
			.saturating_add(ParityDbWeight::get().reads(1_u64))
			.saturating_add(ParityDbWeight::get().writes(1_u64))
	}
	/// Storage: `Messenger::Inbox` (r:1 w:1)
	/// Proof: `Messenger::Inbox` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `Messenger::Channels` (r:1 w:1)
	/// Proof: `Messenger::Channels` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `Messenger::InboxResponses` (r:1 w:1)
	/// Proof: `Messenger::InboxResponses` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `Messenger::CounterForInboxResponses` (r:1 w:1)
	/// Proof: `Messenger::CounterForInboxResponses` (`max_values`: Some(1), `max_size`: Some(4), added: 499, mode: `MaxEncodedLen`)
	/// Storage: `Messenger::MessageWeightTags` (r:1 w:1)
	/// Proof: `Messenger::MessageWeightTags` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `Messenger::UpdatedChannels` (r:1 w:1)
	/// Proof: `Messenger::UpdatedChannels` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	fn relay_message() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `346`
		//  Estimated: `3811`
		// Minimum execution time: 36_673_000 picoseconds.
		Weight::from_parts(37_070_000, 3811)
			.saturating_add(ParityDbWeight::get().reads(6_u64))
			.saturating_add(ParityDbWeight::get().writes(6_u64))
	}
	/// Storage: `Messenger::OutboxResponses` (r:1 w:1)
	/// Proof: `Messenger::OutboxResponses` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `Messenger::Channels` (r:1 w:1)
	/// Proof: `Messenger::Channels` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `Messenger::Outbox` (r:1 w:1)
	/// Proof: `Messenger::Outbox` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `Messenger::CounterForOutbox` (r:1 w:1)
	/// Proof: `Messenger::CounterForOutbox` (`max_values`: Some(1), `max_size`: Some(4), added: 499, mode: `MaxEncodedLen`)
	/// Storage: `Messenger::MessageWeightTags` (r:1 w:1)
	/// Proof: `Messenger::MessageWeightTags` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `Messenger::UpdatedChannels` (r:1 w:1)
	/// Proof: `Messenger::UpdatedChannels` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	fn relay_message_response() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `540`
		//  Estimated: `4005`
		// Minimum execution time: 35_566_000 picoseconds.
		Weight::from_parts(36_336_000, 4005)
			.saturating_add(ParityDbWeight::get().reads(6_u64))
			.saturating_add(ParityDbWeight::get().writes(6_u64))
	}
}
