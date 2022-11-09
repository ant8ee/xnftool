#![cfg(test)]

use super::*;
use codec::Encode;
use cumulus_primitives_core::ParaId;
use frame_support::{assert_err, assert_noop, assert_ok, traits::Currency};
use mock::*;
use orml_traits::{ConcreteFungibleAsset, MultiCurrency};
use polkadot_parachain::primitives::Sibling;
use sp_runtime::{traits::AccountIdConversion, AccountId32};
use xcm_simulator::TestExt;

fn para_a_account() -> AccountId32 {
	ParaId::from(1).into_account_truncating()
}

fn para_b_account() -> AccountId32 {
	ParaId::from(2).into_account_truncating()
}

fn para_d_account() -> AccountId32 {
	ParaId::from(4).into_account_truncating()
}

fn sibling_a_account() -> AccountId32 {
	Sibling::from(1).into_account_truncating()
}

fn sibling_b_account() -> AccountId32 {
	Sibling::from(2).into_account_truncating()
}

fn sibling_c_account() -> AccountId32 {
	Sibling::from(3).into_account_truncating()
}

fn sibling_d_account() -> AccountId32 {
	Sibling::from(4).into_account_truncating()
}

// Not used in any unit tests, but it's super helpful for debugging. Let's
// keep it here.
#[allow(dead_code)]
fn print_events<Runtime: frame_system::Config>(name: &'static str) {
	println!("------ {:?} events -------", name);
	frame_system::Pallet::<Runtime>::events()
		.iter()
		.for_each(|r| println!("> {:?}", r.event));
}

	/// Scenario:
	/// A parachain transfers an NFT resident on the relay chain to another parachain account.
	///
	/// Asserts that the parachain accounts are updated as expected.
	#[test]
	fn withdraw_and_deposit_nft() {
	TestNet::reset();
	Relay::execute_with(|| {
			assert_eq!(relay_chain::Uniques::owner(1, 42), Some(child_account_id(1)));
		});
	ParaA::execute_with(|| {
		assert_noop!(
			ParaXNFTokens::transfer_multiasset(
				Some(ALICE).into(),
				Box::new(X1((GeneralIndex(1), 42u32).into())),
				Box::new(
					(
						Parent,
						Parachain(2),
						Junction::AccountId32 {
							network: NetworkId::Any,
							id: BOB.into()
						}
					)
						.into()
				),
				50,
			),
			Error::<para::Runtime>::AssetHasNoReserve
		);
	});
    Relay::execute_with(|| {
			assert_eq!(relay_chain::Uniques::owner(1, 42), Some(child_account_id(2)));
		});
}

/// Scenario:
	/// The relay-chain teleports an NFT to a parachain.
	///
	/// Asserts that the parachain accounts are updated as expected.
	#[test]
	fn teleport_nft() {
	TestNet::reset();

		Relay::execute_with(|| {
			// Mint the NFT (1, 69) and give it to our "parachain#1 alias".
			assert_ok!(relay_chain::Uniques::mint(
				relay_chain::RuntimeOrigin::signed(ALICE),
				1,
				69,
				child_account_account_id(1, ALICE),
			));
			// The parachain#1 alias of Alice is what must hold it on the Relay-chain for it to be
			// withdrawable by Alice on the parachain.
			assert_eq!(
				relay_chain::Uniques::owner(1, 69),
				Some(child_account_account_id(1, ALICE))
			);
		});
	ParaA::execute_with(|| {
        assert_ok!(parachain::ForeignUniques::force_create(
				parachain::RuntimeOrigin::root(),
				(Parent, GeneralIndex(1)).into(),
				ALICE,
				false,
			));
			assert_eq!(
				parachain::ForeignUniques::owner(((Parent, GeneralIndex(1)).into(), 69u32).into()),
				None,
			);
			assert_eq!(parachain::Balances::reserved_balance(&ALICE), 0);

		assert_noop!(
			ParaXNFTokens::transfer_multiasset(
				Some(ALICE).into(),
				Box::new((GeneralIndex(1), 69u32).into()),
				Box::new(
					MultiLocation::new(
						1,
						X2(
							Parachain(1),
							Junction::AccountId32 {
								network: NetworkId::Any,
								id: BOB.into()
							}
						)
					)
					.into()
				),
				50,
			),
			Error::<para::Runtime>::NotCrossChainTransfer
		);
	});

        ParaA::execute_with(|| {
			assert_eq!(
				parachain::ForeignUniques::owner((Parent, GeneralIndex(1)).into(), 69u32.into()),
				Some(ALICE),
			);
			assert_eq!(parachain::Balances::reserved_balance(&ALICE), 1000);
		});
		Relay::execute_with(|| {
			assert_eq!(relay_chain::Uniques::owner(1, 69), None);
		});
}

/// Scenario:
	/// The relay-chain transfers an NFT into a parachain's sovereign account, who then mints a
	/// trustless-backed-derivated locally.
	///
	/// Asserts that the parachain accounts are updated as expected.
	#[test]
	fn reserve_asset_transfer_nft() {
		sp_tracing::init_for_tests();

	TestNet::reset();

		Relay::execute_with(|| {
			assert_ok!(relay_chain::Uniques::force_create(
				relay_chain::RuntimeOrigin::root(),
				2,
				ALICE,
				false
			));
			assert_ok!(relay_chain::Uniques::mint(
				relay_chain::RuntimeOrigin::signed(ALICE),
				2,
				69,
				child_account_account_id(1, ALICE)
			));
			assert_eq!(
				relay_chain::Uniques::owner(2, 69),
				Some(child_account_account_id(1, ALICE))
			);
		});
	ParaA::execute_with(|| {
assert_ok!(parachain::ForeignUniques::force_create(
				parachain::RuntimeOrigin::root(),
				(Parent, GeneralIndex(2)).into(),
				ALICE,
				false,
			));
			assert_eq!(
				parachain::ForeignUniques::owner((Parent, GeneralIndex(2)).into(), 69u32.into()),
				None,
			);
			assert_eq!(parachain::Balances::reserved_balance(&ALICE), 0);

		assert_noop!(
			ParaXNFTokens::transfer_multiasset(
				Some(ALICE).into(),
				Box::new((GeneralIndex(2), 69u32).into()),
				Box::new(
					MultiLocation::new(
						0,
						X1(Junction::AccountId32 {
							network: NetworkId::Any,
							id: BOB.into()
						})
					)
					.into()
				),
				50,
			),
			Error::<para::Runtime>::InvalidDest
		);
	});
    ParaA::execute_with(|| {
			log::debug!(target: "xcm-exceutor", "Hello");
			assert_eq!(
				parachain::ForeignUniques::owner((Parent, GeneralIndex(2)).into(), 69u32.into()),
				Some(ALICE),
			);
			assert_eq!(parachain::Balances::reserved_balance(&ALICE), 1000);
		});

		Relay::execute_with(|| {
			assert_eq!(relay_chain::Uniques::owner(2, 69), Some(child_account_id(1)));
		});
}

	/// Scenario:
	/// The relay-chain creates an asset class on a parachain and then Alice transfers her NFT into
	/// that parachain's sovereign account, who then mints a trustless-backed-derivative locally.
	///
	/// Asserts that the parachain accounts are updated as expected.
	#[test]
	fn reserve_asset_class_create_and_reserve_transfer() {
		MockNet::reset();

		Relay::execute_with(|| {
			assert_ok!(relay_chain::Uniques::force_create(
				relay_chain::RuntimeOrigin::root(),
				2,
				ALICE,
				false
			));
			assert_ok!(relay_chain::Uniques::mint(
				relay_chain::RuntimeOrigin::signed(ALICE),
				2,
				69,
				child_account_account_id(1, ALICE)
			));
			assert_eq!(
				relay_chain::Uniques::owner(2, 69),
				Some(child_account_account_id(1, ALICE))
			);

			let message = Xcm(vec![Transact {
				origin_kind: OriginKind::Xcm,
				require_weight_at_most: 1_000_000_000,
				call: parachain::RuntimeCall::from(
					pallet_uniques::Call::<parachain::Runtime>::create {
						collection: (Parent, 2u64).into(),
						admin: parent_account_id(),
					},
				)
				.encode()
				.into(),
			}]);
			// Send creation.
			assert_ok!(RelayChainPalletXcm::send_xcm(Here, Parachain(1), message));
		});
		ParaA::execute_with(|| {
			// Then transfer
			let message = Xcm(vec![
				WithdrawAsset((GeneralIndex(2), 69u32).into()),
				DepositReserveAsset {
					assets: AllCounted(1).into(),
					dest: Parachain(1).into(),
					xcm: Xcm(vec![DepositAsset {
						assets: AllCounted(1).into(),
						beneficiary: (AccountId32 { id: ALICE.into(), network: None },).into(),
					}]),
				},
			]);
			let alice = AccountId32 { id: ALICE.into(), network: None };
			assert_ok!(ParachainPalletXcm::send_xcm(alice, Parent, message));
		});
		ParaA::execute_with(|| {
			assert_eq!(parachain::Balances::reserved_balance(&parent_account_id()), 1000);
			assert_eq!(
				parachain::ForeignUniques::collection_owner((Parent, 2u64).into()),
				Some(parent_account_id())
			);
		});
	}

#[test]
fn send_as_sovereign() {
	TestNet::reset();

	Relay::execute_with(|| {
		let _ = RelayBalances::deposit_creating(&para_a_account(), 1_000_000_000_000);
	});

	ParaA::execute_with(|| {
		use xcm::latest::OriginKind::SovereignAccount;

		let call =
			relay::Call::System(frame_system::Call::<relay::Runtime>::remark_with_event { remark: vec![1, 1, 1] });
		let assets: MultiAsset = (Here, 1_000_000_000_000).into();
		assert_ok!(para::OrmlXcm::send_as_sovereign(
			para::Origin::root(),
			Box::new(Parent.into()),
			Box::new(VersionedXcm::from(Xcm(vec![
				WithdrawAsset(assets.clone().into()),
				BuyExecution {
					fees: assets,
					weight_limit: Limited(2_000_000_000)
				},
				Instruction::Transact {
					origin_type: SovereignAccount,
					require_weight_at_most: 1_000_000_000,
					call: call.encode().into(),
				}
			])))
		));
	});

	Relay::execute_with(|| {
		assert!(relay::System::events().iter().any(|r| {
			matches!(
				r.event,
				relay::Event::System(frame_system::Event::<relay::Runtime>::Remarked { sender: _, hash: _ })
			)
		}));
	})
}

#[test]
fn send_as_sovereign_fails_if_bad_origin() {
	TestNet::reset();

	Relay::execute_with(|| {
		let _ = RelayBalances::deposit_creating(&para_a_account(), 1_000_000_000_000);
	});

	ParaA::execute_with(|| {
		use xcm::latest::OriginKind::SovereignAccount;

		let call =
			relay::Call::System(frame_system::Call::<relay::Runtime>::remark_with_event { remark: vec![1, 1, 1] });
		let assets: MultiAsset = (Here, 1_000_000_000_000).into();
		assert_err!(
			para::OrmlXcm::send_as_sovereign(
				para::Origin::signed(ALICE),
				Box::new(Parent.into()),
				Box::new(VersionedXcm::from(Xcm(vec![
					WithdrawAsset(assets.clone().into()),
					BuyExecution {
						fees: assets,
						weight_limit: Limited(10_000_000)
					},
					Instruction::Transact {
						origin_type: SovereignAccount,
						require_weight_at_most: 1_000_000_000,
						call: call.encode().into(),
					}
				])))
			),
			DispatchError::BadOrigin,
		);
	});
}

#[test]
fn call_size_limit() {
	// Ensures Call enum doesn't allocate more than 200 bytes in runtime
	assert!(
		core::mem::size_of::<crate::Call::<crate::tests::para::Runtime>>() <= 200,
		"size of Call is more than 200 bytes: some calls have too big arguments, use Box to \
		reduce the size of Call.
		If the limit is too strong, maybe consider increasing the limit",
	);

	assert!(
		core::mem::size_of::<orml_xcm::Call::<crate::tests::para::Runtime>>() <= 200,
		"size of Call is more than 200 bytes: some calls have too big arguments, use Box to \
		reduce the size of Call.
		If the limit is too strong, maybe consider increasing the limit",
	);
}
