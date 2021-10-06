use super::*;

use crate as kitties;
use frame_support::{assert_noop, assert_ok, parameter_types};
use sp_core::H256;
use sp_runtime::{
    testing::Header,
    traits::{BlakeTwo256, IdentityLookup},
};

// construct mocked test runtime
type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
type Block = frame_system::mocking::MockBlock<Test>;

// Configure a mock runtime to test the pallet.
frame_support::construct_runtime!(
    pub enum Test where
        Block = Block,
        NodeBlock = Block,
        UncheckedExtrinsic = UncheckedExtrinsic,
    {
        System: frame_system::{Pallet, Call, Config, Storage, Event<T>},
        // RandomnessCollectiveFlip: pallet_randomness_collective_flip::{Pallet, Storage},
        Balances: pallet_balances::{Pallet, Call, Storage, Config<T>, Event<T>},
        KittiesModule: kitties::{Pallet, Call, Storage, Event<T>},
    }
);

parameter_types! {
    pub const BlockHashCount: u64 = 250;
    pub const SS58Prefix: u8 = 42;
}

// configure pallets

// configure system pallet
impl frame_system::Config for Test {
    type BaseCallFilter = frame_support::traits::Everything;
    type BlockWeights = ();
    type BlockLength = ();
    type DbWeight = ();
    type Origin = Origin;
    type Call = Call;
    type Index = u64;
    type BlockNumber = u64; // this is just a number for tests, not Substrate ss58 address
    type Hash = H256;
    type Hashing = BlakeTwo256;
    type AccountId = u64;
    type Lookup = IdentityLookup<Self::AccountId>;
    type Header = Header;
    type Event = Event;
    type BlockHashCount = BlockHashCount;
    type Version = ();
    type PalletInfo = PalletInfo;
    type AccountData = pallet_balances::AccountData<u64>;
    type OnNewAccount = ();
    type OnKilledAccount = ();
    type SystemWeightInfo = ();
    type SS58Prefix = SS58Prefix;
    type OnSetCode = ();
}

// configure randomness pallet
// impl pallet_randomness_collective_flip::Config for Test {}

parameter_types! {
    // essentially global variable
  pub static MockRandom: H256 = Default::default ();
}

impl Randomness<H256, u64> for MockRandom {
    fn random(_subject: &[u8]) -> (H256, u64) {
        (MockRandom::get(), 0)
    }
}

// conf balances spallet
parameter_types! {
    pub const ExistentialDeposit: u64 = 1;
}

impl pallet_balances::Config for Test {
    type MaxLocks = ();
    type MaxReserves = ();
    type ReserveIdentifier = ();
    type Balance = u64;
    type Event = Event;
    type DustRemoval = ();
    type ExistentialDeposit = ExistentialDeposit;
    type AccountStore = System;
    type WeightInfo = ();
}

// configure kitties pallet
impl Config for Test {
    type Event = Event;
    type Randomness = MockRandom;
    type KittyIndex = u32;
    type Currency = Balances;
}

// Build genesis storage according to the mock runtime.
pub fn new_test_ext() -> sp_io::TestExternalities {
    let mut t: sp_io::TestExternalities = frame_system::GenesisConfig::default()
        .build_storage::<Test>()
        .unwrap()
        .into();
    // events on block 0 are ignored, so set it to 1
    t.execute_with(|| System::set_block_number(1));
    t
}

#[test]
fn can_create() {
    new_test_ext().execute_with(|| {
        assert_ok!(KittiesModule::create_kitty(Origin::signed(100)));

        let kitty = Kitty([
            59, 250, 138, 82, 209, 39, 141, 109, 163, 238, 183, 145, 235, 168, 18, 122,
        ]);

        assert_eq!(KittiesModule::kitties(100, 0), Some(kitty.clone()));
        assert_eq!(KittiesModule::next_kitty_id(), 1);

        System::assert_last_event(Event::KittiesModule(crate::Event::<Test>::KittyCreated(
            100, 0, kitty,
        )));
    });
}

#[test]
fn gender() {
    assert_eq!(Kitty([0; 16]).gender(), KittyGender::Male);
    assert_eq!(
        Kitty([1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]).gender(),
        KittyGender::Female
    );
}

#[test]
fn can_breed() {
    new_test_ext().execute_with(|| {
        assert_ok!(KittiesModule::create_kitty(Origin::signed(100)));

        // inject to have a different genders of minted kitties
        // System::set_extrinsic_index(1);
        MockRandom::set(H256::from([2; 32]));

        assert_ok!(KittiesModule::create_kitty(Origin::signed(100)));

        // test an error condition
        // asser_nop ensures no state change happened on an tx that errored
        assert_noop!(
            KittiesModule::breed_kitties(Origin::signed(100), 0, 11),
            Error::<Test>::InvalidKittyId
        );
        assert_noop!(
            KittiesModule::breed_kitties(Origin::signed(100), 0, 0),
            Error::<Test>::SameGender
        );
        assert_noop!(
            KittiesModule::breed_kitties(Origin::signed(101), 0, 1),
            Error::<Test>::InvalidKittyId
        );

        assert_ok!(KittiesModule::breed_kitties(Origin::signed(100), 0, 1));

        let kitty = Kitty([
            187, 250, 235, 118, 211, 247, 237, 253, 187, 239, 191, 185, 239, 171, 211, 122,
        ]);

        assert_eq!(KittiesModule::kitties(100, 2), Some(kitty.clone()));
        assert_eq!(KittiesModule::next_kitty_id(), 3);

        System::assert_last_event(Event::KittiesModule(crate::Event::<Test>::KittyBred(
            100u64, 2u32, kitty,
        )));
    });
}
