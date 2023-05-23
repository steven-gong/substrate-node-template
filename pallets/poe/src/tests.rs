use crate::{mock::*, Error, Event};
use frame_support::{assert_noop, assert_ok};
use sp_core::ConstU32;
use sp_runtime::{traits::BadOrigin, BoundedVec};

mod create_claim {
	use super::*;

	#[test]
	fn successful() {
		new_test_ext().execute_with(|| {
			let claim: BoundedVec<u8, ConstU32<10>> = vec![0].try_into().unwrap();

			// Go past genesis block so events get deposited
			System::set_block_number(1);

			// Create a claim.
			assert_ok!(PoeModule::create_claim(RuntimeOrigin::signed(1), claim.clone()));

			// Read pallet storage and assert an expected result.
			assert_eq!(PoeModule::proofs(claim.clone()), Some((1, 1)));

			// Assert that the correct event was deposited
			System::assert_last_event(Event::ClaimCreated { creator: 1, claim }.into());
		});
	}

	mod failed_when {
		use super::*;

		#[test]
		fn bad_origin() {
			new_test_ext().execute_with(|| {
				let claim: BoundedVec<u8, ConstU32<10>> = vec![0].try_into().unwrap();

				System::set_block_number(1);

				// Root origin revokes claim
				assert_noop!(
					PoeModule::create_claim(RuntimeOrigin::root(), claim.clone()),
					BadOrigin
				);

				// None origin revokes claim
				assert_noop!(
					PoeModule::create_claim(RuntimeOrigin::none(), claim.clone()),
					BadOrigin
				);
			});
		}

		#[test]
		fn claim_already_exists() {
			new_test_ext().execute_with(|| {
				let claim: BoundedVec<u8, ConstU32<10>> = vec![0].try_into().unwrap();

				System::set_block_number(1);

				// Create a claim
				let _ = PoeModule::create_claim(RuntimeOrigin::signed(1), claim.clone());

				// Create the same claim again
				assert_noop!(
					PoeModule::create_claim(RuntimeOrigin::signed(1), claim.clone()),
					Error::<Test>::ProofAlreadyExists
				);
			});
		}
	}
}

mod revoke_claim {
	use super::*;

	#[test]
	fn successful() {
		new_test_ext().execute_with(|| {
			let claim: BoundedVec<u8, ConstU32<10>> = vec![0].try_into().unwrap();

			System::set_block_number(1);

			// Dispatch a signed extrinsic.
			let _ = PoeModule::create_claim(RuntimeOrigin::signed(1), claim.clone());

			// Sign and call the extrinsic
			assert_ok!(PoeModule::revoke_claim(RuntimeOrigin::signed(1), claim.clone()));

			// Assert that the claim was removed from the storage.
			assert_eq!(PoeModule::proofs(claim.clone()), None);

			// Assert that the correct event was deposited
			System::assert_last_event(Event::ClaimRevoked { owner: 1, claim }.into());
		});
	}

	mod failed_when {
		use super::*;

		#[test]
		fn bad_origin() {
			new_test_ext().execute_with(|| {
				let claim: BoundedVec<u8, ConstU32<10>> = vec![0].try_into().unwrap();

				System::set_block_number(1);

				let _ = PoeModule::create_claim(RuntimeOrigin::signed(1), claim.clone());

				// Root origin revokes claim
				assert_noop!(
					PoeModule::revoke_claim(RuntimeOrigin::root(), claim.clone()),
					BadOrigin
				);

				// None origin revokes claim
				assert_noop!(
					PoeModule::revoke_claim(RuntimeOrigin::none(), claim.clone()),
					BadOrigin
				);
			});
		}

		#[test]
		fn claim_not_exist() {
			new_test_ext().execute_with(|| {
				let claim: BoundedVec<u8, ConstU32<10>> = vec![0].try_into().unwrap();

				System::set_block_number(1);

				// Revoke a claim that is never been created
				assert_noop!(
					PoeModule::revoke_claim(RuntimeOrigin::signed(1), claim.clone()),
					Error::<Test>::ClaimNotExist
				);
			});
		}

		#[test]
		fn caller_is_not_owner() {
			new_test_ext().execute_with(|| {
				let claim: BoundedVec<u8, ConstU32<10>> = vec![0].try_into().unwrap();

				System::set_block_number(1);

				// Account 1 creates claim
				let _ = PoeModule::create_claim(RuntimeOrigin::signed(1), claim.clone());

				// Account 2 revokes claim
				assert_noop!(
					PoeModule::revoke_claim(RuntimeOrigin::signed(2), claim.clone()),
					Error::<Test>::NotClaimOwner
				);
			});
		}
	}
}

mod transfer_claim {
	use super::*;

	#[test]
	fn successful() {
		new_test_ext().execute_with(|| {
			let claim: BoundedVec<u8, ConstU32<10>> = vec![0].try_into().unwrap();

			System::set_block_number(1);

			// Account 1 creates a claim
			let _ = PoeModule::create_claim(RuntimeOrigin::signed(1), claim.clone());

			// Account 1 transfer this claim to account 2
			assert_ok!(PoeModule::transfer_claim(RuntimeOrigin::signed(1), claim.clone(), 2));

			// Assert that the ownership of claim was updated to account 2
			assert_eq!(PoeModule::proofs(claim.clone()), Some((2, 1)));

			// Assert that the correct event was deposited
			System::assert_last_event(
				Event::ClaimTransferred { sender: 1, receiver: 2, claim }.into(),
			);
		});
	}

	mod failed_when {
		use super::*;

		#[test]
		fn bad_origin() {
			new_test_ext().execute_with(|| {
				let claim: BoundedVec<u8, ConstU32<10>> = vec![0].try_into().unwrap();

				System::set_block_number(1);

				let _ = PoeModule::create_claim(RuntimeOrigin::signed(1), claim.clone());

				// Root origin transfers claim
				assert_noop!(
					PoeModule::transfer_claim(RuntimeOrigin::root(), claim.clone(), 2),
					BadOrigin
				);

				// None origin transfers claim
				assert_noop!(
					PoeModule::transfer_claim(RuntimeOrigin::none(), claim.clone(), 2),
					BadOrigin
				);
			});
		}

		#[test]
		fn claim_not_exist() {
			new_test_ext().execute_with(|| {
				let claim: BoundedVec<u8, ConstU32<10>> = vec![0].try_into().unwrap();

				System::set_block_number(1);

				// Transfer a claim that is never been created
				assert_noop!(
					PoeModule::transfer_claim(RuntimeOrigin::signed(1), claim.clone(), 2),
					Error::<Test>::ClaimNotExist
				);
			});
		}

		#[test]
		fn sender_is_not_owner() {
			new_test_ext().execute_with(|| {
				let claim: BoundedVec<u8, ConstU32<10>> = vec![0].try_into().unwrap();

				System::set_block_number(1);

				// Account 1 creates a claim
				let _ = PoeModule::create_claim(RuntimeOrigin::signed(1), claim.clone());

				// Account 3 transfer this claim to account 2
				assert_noop!(
					PoeModule::transfer_claim(RuntimeOrigin::signed(3), claim.clone(), 2),
					Error::<Test>::NotClaimOwner
				);
			});
		}
	}
}
