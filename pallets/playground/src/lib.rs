#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;
use codec::FullCodec;
use core::fmt::Debug;
use scale_info::TypeInfo;
use sp_std::vec::Vec;
use frame_support::{pallet_prelude::*, traits::OriginTrait};
use frame_system::pallet_prelude::*;
use pallet_best_path::{best_path, traits::BestPath, types::ProviderPairOperation};
use pallet_scheduler_datetime::traits::*;
use sp_runtime::{traits::Dispatchable};

pub type CallOf<T> = <T as Config>::Call;
pub type PalletsOriginOf<T> = <<T as frame_system::Config>::Origin as OriginTrait>::PalletsOrigin;

/// Pallet used to test integration with other third party pallets such as `pallet_best_path` and `pallet_schedule_datetime`.
/// 
/// The aim is to provide usage of other pallets, either by calling them (pallet_best_path/pallet_schedule_datetime)
/// or letting them call us back (via pallet_schedule_datetime).
/// As such, this pallet circumnavigates Origin security checks imposed by the original pallets ;).
/// 
/// Usage:
/// - setup pallet_best_path as per instructions in https://github.com/konrads/pallet-best_path
/// - whitelist the OCW user
/// - `make populate-keys`
/// - `submit_monitored_pairs()` - delegates to pallet_best_path (can also do from pallet_best_path)
/// - `schedule_monitoring()` - delegates to pallet_scheduler_datetime, which calls back `lookup_price()`
/// 
/// __NOTE:__ pallet_schedule_datetime requires PalletOrigin setup. Research for that was based on democracy and referenda pallets,
/// both of which schedule internally. Current implementation delegates to Root to perform scheduled calls, should consider changing to original caller.
#[frame_support::pallet]
pub mod pallet {
	use super::*;
	
	#[pallet::error]
	pub enum Error<T> {
		/// Indicates failure in scheduling of the call
		FailedToSchedule
	}

	#[pallet::config]
	pub trait Config: frame_system::Config where <Self as Config>::Call: From<Call<Self>> {
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
		type Call: Parameter + Dispatchable<Origin = Self::Origin> + From<Call<Self>>;
		type PalletsOrigin: From<frame_system::RawOrigin<Self::AccountId>>;

		type Currency: best_path::prelude::Currency + FullCodec + TypeInfo + Debug + AsRef<[u8]>;
		type Provider: best_path::prelude::Provider + FullCodec + TypeInfo + Debug;
		type Amount: best_path::prelude::Amount + Debug + Eq;

		type BestPath: BestPath<Self::Currency, Self::Amount, Self::Provider>;
		type Scheduler: Named<Self::BlockNumber, CallOf<Self>, PalletsOriginOf<Self>>;
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

		/// Acknowledgement of version update
		PostUpdateAck,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Extrinsic useful in eg. scheduler/best path testing.
		#[pallet::weight(10_000)]
		pub fn hello(origin: OriginFor<T>) -> DispatchResult {
			let who = ensure_signed_or_root(origin)?;
			let now = <frame_system::Pallet<T>>::block_number();
			log::info!(target: "runtime::playground", "Hello @ block {:?} from signed or root origin {:?}", now, who);
			if let Some(who) = who {
				Self::deposit_event(Event::Hello(now, who));
			} else {
				Self::deposit_event(Event::HelloRoot(now));
			}
			Ok(())
		}

		/// Submits call to best_path pallet
		#[pallet::weight(10_000)]
		pub fn submit_monitored_pairs(origin: OriginFor<T>, operations: Vec<ProviderPairOperation<T::Currency, T::Provider>>) -> DispatchResult {
			ensure_signed(origin)?;
			T::BestPath::submit_monitored_pairs(operations);
			Ok(())
		}

		/// Submits call to schedule_datetime pallet, requests scheduling of lookup_price() extrinsic
		#[pallet::weight(10_000)]
		pub fn schedule_monitoring(origin: OriginFor<T>, source: T::Currency, target: T::Currency, schedule: Schedule) -> DispatchResult {
			ensure_signed(origin)?;
			let mut schedule_id = source.as_ref().to_vec();
			schedule_id.push(b'-');
			schedule_id.extend(target.as_ref().iter());
			T::Scheduler::schedule_named(
				schedule_id,
				schedule,
				200,
				frame_system::RawOrigin::Root.into(), // FIXME: change to Signed root, eg. along the lines of: frame_system::RawOrigin::Signed(who).into(),
				MaybeHashed::Value(Call::lookup_price { source, target }.into()),
			).map_err(|_| Error::<T>::FailedToSchedule)?;
			Ok(())
		}

		/// Called by the scheduler_datetime pallet
		#[pallet::weight(10_000)]
		pub fn lookup_price(origin: OriginFor<T>, source: T::Currency, target: T::Currency) -> DispatchResult {
			ensure_signed_or_root(origin)?;
			let source_str = sp_std::str::from_utf8(&source.as_ref()).unwrap();
			let target_str = sp_std::str::from_utf8(&target.as_ref()).unwrap();
			let price_path = T::BestPath::get_price_path(source.clone(), target.clone());
			log::info!(target: "runtime::playground", "looked up price {} -> {}: {:?}", source_str, target_str, &price_path);
			Self::deposit_event(Event::PricePathLookup(source, target, price_path));
			Ok(())
		}

		#[pallet::weight(10_000)]
		pub fn post_update_ack(origin: OriginFor<T>) -> DispatchResult {
			ensure_signed(origin)?;
			Self::deposit_event(Event::PostUpdateAck);
			Ok(())
		}
	}
}
