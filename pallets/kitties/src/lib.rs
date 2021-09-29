#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::pallet_prelude::*;
use frame_support::traits::Randomness;
use frame_system::{ensure_signed, pallet_prelude::OriginFor};
use sp_io::hashing::blake2_128;
use sp_runtime::ArithmeticError;
use sp_std::result::Result;

pub use pallet::*;

// only included for the test build
#[cfg(test)]
mod tests;

#[derive(Encode, Decode, RuntimeDebug, Eq, PartialEq, Clone)]
pub struct Kitty(pub [u8; 16]);

#[derive(Encode, Decode, RuntimeDebug, Eq, PartialEq, Clone)]
pub enum KittyGender {
    Male,
    Female,
}

impl Kitty {
    pub fn gender(&self) -> KittyGender {
        if self.0[0] % 2 == 0 {
            KittyGender::Male
        } else {
            KittyGender::Female
        }
    }
}

#[frame_support::pallet]
pub mod pallet {

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
        // a kitty is created \[owner, kitty_id, kitty\]
        KittyCreated(T::AccountId, u32, Kitty),
        // a kitty is bred \[owner, kitty_id, kitty\]
        KittyBred(T::AccountId, u32, Kitty),
    }

    #[pallet::error]
    pub enum Error<T> {
        InvalidKittyId,
        SameGender,
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Create a new kitty
        #[pallet::weight(1000)]
        pub fn create_kitty(origin: OriginFor<T>) -> DispatchResult {
            let sender = ensure_signed(origin)?;
            let dna = Self::random_value(&sender);

            // Create and store kitty
            let kitty = Kitty(dna);
            let next_kitty_id: u32 = Self::get_next_kitty_id()?;

            Kitties::<T>::insert(&sender, next_kitty_id, &kitty);

            // Emit an event
            Self::deposit_event(Event::KittyCreated(sender, next_kitty_id, kitty));

            Ok(())
        }

        /// Breed kitties
        #[pallet::weight(1000)]
        pub fn breed_kitties(
            origin: OriginFor<T>,
            kitty_id_1: u32,
            kitty_id_2: u32,
        ) -> DispatchResult {
            let sender = ensure_signed(origin)?;

            let kitty1 = Self::kitties(&sender, kitty_id_1).ok_or(Error::<T>::InvalidKittyId)?;
            let kitty2 = Self::kitties(&sender, kitty_id_2).ok_or(Error::<T>::InvalidKittyId)?;

            ensure!(kitty1.gender() != kitty2.gender(), Error::<T>::SameGender);

            let next_kitty_id: u32 = Self::get_next_kitty_id()?;

            let kitty1_dna = kitty1.0;
            let kitty2_dna = kitty2.0;

            let selector = Self::random_value(&sender);

            let mut new_dna = [0u8; 16];
            // Combine parents and selector to create new kitty
            for i in 0..kitty1_dna.len() {
                new_dna[i] = combine_dna(kitty1_dna[i], kitty2_dna[i], selector[i]);
            }

            let new_kitty = Kitty(new_dna);

            Kitties::<T>::insert(&sender, next_kitty_id, &new_kitty);

            Self::deposit_event(Event::KittyBred(sender, next_kitty_id, new_kitty));

            Ok(())
        }
    }

    fn combine_dna(dna1: u8, dna2: u8, selector: u8) -> u8 {
        // selector[bit_index] == 0 -> use dna1[bit_index]
        // selector[bit_index] == 1 -> use dna2[bit_index]
        // e.g.
        // selector     = 0b00000001
        // dna1		= 0b10101010
        // dna2		= 0b00001111
        // result	= 0b10101011

        (!selector & dna1) | (selector & dna2)
    }
}

impl<T: Config> Pallet<T> {
    // fn get_next_kitty_id() -> Result<u32, DispatchError> {
    //     NextKittyId::<T>::try_mutate(|current_id_ptr| -> Result<u32, DispatchError> {
    //         let current_id = *current_id_ptr;
    //         *current_id_ptr = current_id.checked_add(1).ok_or(ArithmeticError::Overflow)?;
    //         Ok(current_id)
    //     })
    // }

    fn get_next_kitty_id() -> Result<u32, DispatchError> {
        NextKittyId::<T>::try_mutate(|next_id| -> Result<u32, DispatchError> {
            let current_id = *next_id;
            *next_id = next_id.checked_add(1).ok_or(ArithmeticError::Overflow)?;
            Ok(current_id)
        })
    }

    /// Generate a random 128bit value
    fn random_value(sender: &T::AccountId) -> [u8; 16] {
        let payload =
            (
                <pallet_randomness_collective_flip::Pallet<T> as Randomness<
                    T::Hash,
                    T::BlockNumber,
                >>::random_seed()
                .0,
                &sender,
                <frame_system::Pallet<T>>::extrinsic_index(),
            );

        payload.using_encoded(blake2_128)
    }
}
