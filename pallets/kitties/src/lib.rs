#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::pallet_prelude::*;
pub use pallet::*;
use sp_io::hashing::blake2_128;
use sp_runtime::ArithmeticError;

#[derive(Encode, Decode, RuntimeDebug, Eq, PartialEq, Clone)]
pub struct Kitty(pub [u8; 16]);

#[frame_support::pallet]
pub mod pallet {
	use frame_support::traits::Randomness;
	use frame_system::{ensure_signed, pallet_prelude::OriginFor};

	use super::*;

	#[pallet::config]
	pub trait Config: frame_system::Config + pallet_randomness_collective_flip::Config {
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
	}

	// stores all the kitties. Key is (user, kitty_id), value is Kitty
	#[pallet::storage]
	#[pallet::getter(fn kitties)]
	pub(super) type Kitties<T: Config> = StorageDoubleMap<
		_,
		Blake2_128Concat,
		T::AccountId,
		Blake2_128Concat,
		u32,
		Kitty,
		OptionQuery, // returns Option<Kitty>
	>;

	// stores the next kitty id
	#[pallet::storage]
	#[pallet::getter(fn next_kitty_id)]
	pub(super) type NextKittyId<T: Config> = StorageValue<_, u32, ValueQuery>;

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	// T - runtime type which implements the Config
	pub struct Pallet<T>(_);

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	#[pallet::metadata(T::AccountId = "AccountId")]
	pub enum Event<T: Config> {
		// a kitty is created [owner, kitty_id, kitty]
		KittyCreated(T::AccountId, u32, Kitty),
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight(1000)]

		/// Create a new kitty
		pub fn create_kitty(origin: OriginFor<T>) -> DispatchResult {
			let sender = ensure_signed(origin)?;

			// Generate a random 128bit value
			let payload = (
				<pallet_randomness_collective_flip::Pallet<T> as Randomness<
					T::Hash,
					T::BlockNumber,
				>>::random_seed()
				.0,
				&sender,
				<frame_system::Pallet<T>>::extrinsic_index(),
			);

			let dna = payload.using_encoded(blake2_128);

			// Create and store kitty
			let kitty = Kitty(dna);
			let kitty_id = Self::next_kitty_id();
			Kitties::<T>::insert(&sender, kitty_id, kitty.clone());

			// NOTE: ensures kitty id does not overflow
			let next_kitty_id: u32 = match kitty_id.checked_add(1) {
				None => return Err(ArithmeticError::Overflow.into()),
				Some(id) => id,
			};

			NextKittyId::<T>::put(next_kitty_id);

			// Emit an event
			Self::deposit_event(Event::KittyCreated(sender, kitty_id, kitty));

			Ok(())
		}
	}
}
