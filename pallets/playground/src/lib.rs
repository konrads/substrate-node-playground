#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;
use codec::FullCodec;
use sp_std::vec::Vec;
use core::fmt::Debug;
use frame_support::pallet_prelude::*;
use frame_system::pallet_prelude::*;
use pallet_best_path::{best_path, traits::BestPath, types::ProviderPairOperation};

/// Pallet used to test integration with other third party pallets such as pallet_best_path and pallet_schedule_datetime.
#[frame_support::pallet]
pub mod pallet {
	use super::*;

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

		type Currency: best_path::prelude::Currency + FullCodec + TypeInfo + Debug + AsRef<[u8]>;
		type Provider: best_path::prelude::Provider + FullCodec + TypeInfo + Debug;
		type Amount: best_path::prelude::Amount + Debug + Eq;

		type BestPath: BestPath<Self::Currency, Self::Amount, Self::Provider>;
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Hello from non root origin
		/// parameters. [when, who]
		Hello(T::BlockNumber, T::AccountId),

		/// Hello from a Root origin.
		/// parameters. [when]
		HelloRoot(T::BlockNumber),

		/// Price Pair updated
		/// parameters. [source, target, price_path]
		PricePathLookup(T::Currency, T::Currency, Option<best_path::prelude::PricePath<T::Currency, T::Amount, T::Provider>>),
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Extrinsic useful in eg. scheduler testing.
		#[pallet::weight(10_000)]
		pub fn hello(origin: OriginFor<T>) -> DispatchResult {
			let who = ensure_signed_or_root(origin)?;
			let now = <frame_system::Pallet<T>>::block_number();
			log::warn!(target: "runtime::playground", "Hello @ block {:?} from signed or root origin {:?}", now, who);
			if let Some(who) = who {
				Self::deposit_event(Event::Hello(now, who));
			} else {
				Self::deposit_event(Event::HelloRoot(now));
			}
			Ok(())
		}

		#[pallet::weight(10_000)]
		pub fn monitored_pairs(origin: OriginFor<T>, operations: Vec<ProviderPairOperation<T::Currency, T::Provider>>) -> DispatchResult {
			ensure_signed(origin)?;
			T::BestPath::submit_monitored_pairs(operations);
			Ok(())
		}

		#[pallet::weight(10_000)]
		pub fn lookup_price(origin: OriginFor<T>, source: T::Currency, target: T::Currency) -> DispatchResult {
			ensure_signed(origin)?;
			Self::deposit_event(Event::PricePathLookup(source.clone(), target.clone(), T::BestPath::get_price_path(source, target)));
			Ok(())
		}
	}
}
