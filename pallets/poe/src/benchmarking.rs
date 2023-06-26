use crate::*;
use frame_benchmarking::{account, benchmarks, whitelisted_caller};
use frame_support::sp_std::vec;
use frame_system::RawOrigin;

const SEED: u32 = 0;

benchmarks! {
	create_claim {
		let d in 0 .. T::MaxClaimLength::get();
		let claim = BoundedVec::try_from(vec![0; d as usize]).unwrap();
		let caller: T::AccountId = whitelisted_caller();
	} : _(RawOrigin::Signed(caller.clone()), claim.clone())
	verify {
		assert_eq!(
			Proofs::<T>::get(&claim),
			Some((caller, frame_system::Pallet::<T>::block_number()))
		);
	}

	revoke_claim {
		let d in 0 .. T::MaxClaimLength::get();
		let claim = BoundedVec::try_from(vec![0; d as usize]).unwrap();
		let caller: T::AccountId = whitelisted_caller();
		Proofs::<T>::insert(
			&claim,
			(caller.clone(), frame_system::Pallet::<T>::block_number()),
		);
	} : _(RawOrigin::Signed(caller.clone()), claim.clone())
	verify {
		assert!(!Proofs::<T>::contains_key(&claim));
	}

	transfer_claim {
		let d in 0 .. T::MaxClaimLength::get();
		let claim = BoundedVec::try_from(vec![0; d as usize]).unwrap();
		let caller: T::AccountId = whitelisted_caller();
		let receiver: T::AccountId = account("receiver", 0, SEED);
		Proofs::<T>::insert(
			&claim,
			(caller.clone(), frame_system::Pallet::<T>::block_number()),
		);
	} : _(RawOrigin::Signed(caller.clone()), claim.clone(), receiver.clone())
	verify {
		assert_eq!(
			Proofs::<T>::get(&claim),
			Some((receiver, frame_system::Pallet::<T>::block_number()))
		);
	}

	impl_benchmark_test_suite!(PoeModule, crate::mock::new_test_ext(), crate::mock::Test);
}
