#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::*;

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Event documentation `hello`
		/// parameters. [when, who]
		Hello(T::BlockNumber, T::AccountId),

		/// Event documentation `helloRoot`
		/// parameters. [when]
		HelloRoot(T::BlockNumber),
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
	}
}
