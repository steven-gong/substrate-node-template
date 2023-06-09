use crate::{Config, Kitties, Kitty, KittyId, Pallet};
use frame_support::{
	log, migration::storage_key_iter, pallet_prelude::*, storage::StoragePrefixedMap,
	traits::GetStorageVersion, weights::Weight, Blake2_128Concat,
};

#[derive(
	Encode, Decode, Clone, Copy, RuntimeDebug, PartialEq, Eq, Default, TypeInfo, MaxEncodedLen,
)]
pub struct V0Kitty(pub [u8; 16]);

#[derive(
	Encode, Decode, Clone, Copy, RuntimeDebug, PartialEq, Eq, Default, TypeInfo, MaxEncodedLen,
)]
pub struct V1Kitty {
	pub dna: [u8; 16],
	pub name: [u8; 4],
}

pub fn migrate<T: Config>() -> Weight {
	let on_chain_storage_version = Pallet::<T>::on_chain_storage_version();
	let current_storage_version = Pallet::<T>::current_storage_version();
	log::info!(
		target: "runtime::kitties",
		"Running migration to v2 for kitties with current storage version {:?} / onchain {:?}",
		current_storage_version,
		on_chain_storage_version,
	);

	if on_chain_storage_version > 1 || current_storage_version != 2 {
		return Weight::zero();
	}

	if current_storage_version == 0 {
		v0_to_v2::<T>();
	} else if current_storage_version == 1 {
		v1_to_v2::<T>();
	} else {
		log::warn!(
			target: "runtime::kitties",
			"Attempted to apply migration to v2 but failed because on chain storage version is {:?}",
			on_chain_storage_version,
		);
	}

	Weight::zero()
}

fn v0_to_v2<T: Config>() {
	let module = Kitties::<T>::module_prefix();
	let item = Kitties::<T>::storage_prefix();
	for (index, kitty) in
		storage_key_iter::<KittyId, V0Kitty, Blake2_128Concat>(module, item).drain()
	{
		let new_kitty = Kitty { dna: kitty.0, name: *b"abcd0000" };
		Kitties::<T>::insert(index, &new_kitty);
		log::info!(
			target: "runtime::kitties",
			"kitty `{:?}` is migrated from v0 to v2",
			new_kitty.name,
		);
	}
}

fn v1_to_v2<T: Config>() {
	let module = Kitties::<T>::module_prefix();
	let item = Kitties::<T>::storage_prefix();
	for (index, kitty) in
		storage_key_iter::<KittyId, V1Kitty, Blake2_128Concat>(module, item).drain()
	{
		let mut new_name: [u8; 8] = [0; 8];

		new_name[..kitty.name.len()].copy_from_slice(&kitty.name);
		new_name[kitty.name.len()..].copy_from_slice(&*b"0000");

		let new_kitty = Kitty { dna: kitty.dna, name: new_name };
		Kitties::<T>::insert(index, &new_kitty);

		log::info!(
			target: "runtime::kitties",
			"kitty `{:?}` is migrated from v1 to v2",
			kitty.name,
		);
	}
}
