#![allow(unused_imports)]
#![allow(dead_code)]
#![allow(unused_variables)]
#![cfg_attr(not(feature = "std"), no_std)]

use core::marker::PhantomData;
use cumulus_primitives_core::ParaId;
use frame_support::{
	pallet_prelude::*,
	traits::{Currency, Get, Hooks, IsType, Randomness, ReservableCurrency},
	transactional, PalletId,
};
use frame_system::{
	ensure_root, ensure_signed,
	pallet_prelude::{BlockNumberFor, OriginFor},
};
use sp_runtime::{
	traits::{Saturating, Zero},
	DispatchResult, Permill, SaturatedConversion,
};
use sp_std::{boxed::Box, vec, vec::Vec};
use xcm::v0::prelude::*;
use xcm::v0::{
	Error as XcmError, ExecuteXcm, Junction, MultiAsset, MultiLocation, Order, Outcome,
	Result as XcmResult, Xcm,
};

use xcm_executor::{
	traits::{Convert, FilterAssetLocation, TransactAsset, WeightBounds},
	Assets,
};

pub use pallet::*;
// Log filter
const MANTA_XASSETS: &str = "manta-xassets";
pub type BalanceOf<T> =
	<<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

#[frame_support::pallet]
pub mod pallet {
	use super::*;

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
		type XcmExecutor: ExecuteXcm<Self::Call>;
		/// The type used to actually dispatch an XCM to its destination.
		type XcmRouter: SendXcm;
		type FriendChains: Get<Vec<(MultiLocation, u128)>>;
		type Conversion: Convert<MultiLocation, Self::AccountId>;
		type Currency: ReservableCurrency<Self::AccountId>;
		type SelfParaId: Get<ParaId>;
		/// Means of measuring the weight consumed by an XCM message locally.
		type Weigher: WeightBounds<Self::Call>;
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	// pub struct Pallet<T>(PhantomData<T>);
	pub struct Pallet<T>(_);

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T> {}

	#[pallet::error]
	pub enum Error<T> {
		BalanceLow,
		SelfChain,
		UnweighableMessage,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// transfer to parachain
		#[pallet::weight(10000)]
		fn transfer_me_sir(
			origin: OriginFor<T>,
			dest: T::AccountId,
			message: Xcm<()>,
		) -> DispatchResult {
			log::info! {target: MANTA_XASSETS, "\n \n \n \n \n \n \n \n \n \n \n \n \n \n \n \n \n \n \n \n \n \n \n START transfer_me_sir() "};

			let from = ensure_signed(origin)?;
			log::info! {target: MANTA_XASSETS, "\n \n from_origin = {:?} \n \n", from};

			let xcm_origin = T::Conversion::reverse(from).expect("failed to create xcm origin");
			log::info! {target: MANTA_XASSETS, "\n \n xcm_origin = {:?} \n \n", xcm_origin};

			let xcm_target =
				T::Conversion::reverse(dest.clone()).expect("failed to create xcm target");
			log::info! {target: MANTA_XASSETS, "\n \n xcm_target = {:?} \n \n", xcm_target};

			log::info! {target: MANTA_XASSETS, "\n \n RIGHT BEFORE SEND_XCM \n \n "};

			T::XcmRouter::send_xcm(xcm_target, message).unwrap();

			log::info! {target: MANTA_XASSETS, "NO ERROR \n \n \n \n \n \n \n \n \n \n \n \n \n \n \n \n \n \n \n \n \n \n \n NO ERROR "};

			Ok(().into())
		}

		/// transfer to relaychain
		#[pallet::weight(10000)]
		fn transfer_to_relaychain(
			origin: OriginFor<T>,
			dest: T::AccountId,
			#[pallet::compact] amount: BalanceOf<T>,
		) -> DispatchResult {
			let from = ensure_signed(origin)?;

			// create friend relaychain target
			let xcm_target = T::Conversion::reverse(dest.clone())
				.expect("failed to create friend chain target origin");

			// friend chain location
			let asset_location = MultiLocation::X1(Junction::Parent);

			log::info! {target: MANTA_XASSETS, "amount = {:?}", amount};
			log::info! {target: MANTA_XASSETS, "asset_location = {:?}", asset_location};

			let amount = amount.saturated_into::<u128>();

			// create friend relaychain xcm
			let friend_xcm = Xcm::<T>::WithdrawAsset {
				assets: vec![MultiAsset::ConcreteFungible {
					id: asset_location.clone(),
					amount,
				}],
				effects: vec![Order::DepositAsset {
					assets: vec![MultiAsset::All],
					dest: asset_location,
				}],
			};

			log::info! {target: MANTA_XASSETS, "friend_xcm = {:?}", friend_xcm};

			let xcm_outcome = T::XcmExecutor::execute_xcm(xcm_target, friend_xcm.into(), 300_0000);

			log::info! {target: MANTA_XASSETS, "xcm_outcome = {:?}", xcm_outcome};

			Ok(().into())
		}
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

	#[pallet::genesis_config]
	pub struct GenesisConfig<T: Config> {
		pub place_holder: PhantomData<T>,
	}

	#[cfg(feature = "std")]
	impl<T: Config> Default for GenesisConfig<T> {
		fn default() -> GenesisConfig<T> {
			Self {
				place_holder: PhantomData,
			}
		}
	}

	#[pallet::genesis_build]
	impl<T: Config> GenesisBuild<T> for GenesisConfig<T> {
		fn build(&self) {}
	}

	impl<T: Config> Pallet<T> {
		/// Relay an XCM `message` from a given `interior` location in this context to a given `dest`
		/// location. A null `dest` is not handled.
		pub fn send_xcm(
			interior: MultiLocation,
			dest: MultiLocation,
			message: Xcm<()>,
		) -> Result<(), XcmError> {
			let message = match interior {
				MultiLocation::Null => message,
				who => Xcm::<()>::RelayedFrom {
					who,
					message: Box::new(message),
				},
			};
			T::XcmRouter::send_xcm(dest, message)
		}
	}
}
