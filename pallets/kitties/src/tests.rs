use crate::{mock::*, Error, Event};
use frame_support::{assert_noop, assert_ok};
use sp_runtime::{
	traits::BadOrigin,
	DispatchError::Token,
	TokenError::{FundsUnavailable, NotExpendable},
};

const KITTY_A: [u8; 4] = *b"ktyA";
const KITTY_B: [u8; 4] = *b"ktyB";
const KITTY_C: [u8; 4] = *b"ktyC";

mod create_kitty {
	use super::*;

	#[test]
	fn successful() {
		new_test_ext().execute_with(|| {
			let creator_initial_balance = Balances::free_balance(&VALID_KITTY_CREATOR);
			let kitties_account_initial_balance = Balances::free_balance(&Kitties::account_id());

			let kitty_id = 0;
			assert_eq!(Kitties::next_kitty_id(), kitty_id);

			assert_ok!(Kitties::create(RuntimeOrigin::signed(VALID_KITTY_CREATOR), KITTY_A));

			assert_eq!(
				creator_initial_balance - KittyPrice::get(),
				Balances::free_balance(&VALID_KITTY_CREATOR)
			);
			assert_eq!(
				kitties_account_initial_balance + KittyPrice::get(),
				Balances::free_balance(&Kitties::account_id())
			);

			assert_eq!(Kitties::next_kitty_id(), kitty_id + 1);
			assert!(Kitties::kitties(kitty_id).is_some());
			assert_eq!(Kitties::kitty_owner(kitty_id), Some(VALID_KITTY_CREATOR));
			assert_eq!(Kitties::kitty_parents(kitty_id), None);

			let kitty = Kitties::kitties(kitty_id).expect("Kitty was created");
			System::assert_last_event(
				Event::KittyCreated { who: VALID_KITTY_CREATOR, kitty_id, kitty }.into(),
			);
		});
	}

	mod failed_when {
		use super::*;

		#[test]
		fn bad_origin() {
			new_test_ext().execute_with(|| {
				assert_noop!(Kitties::create(RuntimeOrigin::root(), KITTY_A), BadOrigin);
				assert_noop!(Kitties::create(RuntimeOrigin::none(), KITTY_A), BadOrigin);
			});
		}

		#[test]
		fn next_kitty_id_overflow() {
			new_test_ext().execute_with(|| {
				crate::NextKittyId::<Test>::set(crate::KittyId::MAX);
				assert_noop!(
					Kitties::create(RuntimeOrigin::signed(VALID_KITTY_CREATOR), KITTY_A),
					Error::<Test>::KittyIdCannotOverflow
				);
			});
		}

		#[test]
		fn not_enough_fund() {
			new_test_ext().execute_with(|| {
				let kitty_id = 0;
				assert_eq!(Kitties::next_kitty_id(), kitty_id);

				assert_noop!(
					Kitties::create(
						RuntimeOrigin::signed(ACCOUNT_WITH_ONLY_EXISTENTIAL_DEPOSIT),
						KITTY_A
					),
					Token(FundsUnavailable)
				);
			});
		}

		#[test]
		fn not_able_to_keep_alive() {
			new_test_ext().execute_with(|| {
				let kitty_id = 0;
				assert_eq!(Kitties::next_kitty_id(), kitty_id);

				assert_noop!(
					Kitties::create(
						RuntimeOrigin::signed(ACCOUNT_WITH_JUST_KITTY_PRICE_AMOUNT),
						KITTY_A
					),
					Token(NotExpendable)
				);
			});
		}
	}
}

mod breed_kitty {
	use super::*;

	#[test]
	fn successful() {
		new_test_ext().execute_with(|| {
			let kitty_id_1 = 0;
			let kitty_id_2 = 1;
			let bred_kitty_id = 2;

			let creator_initial_balance = Balances::free_balance(&VALID_KITTY_CREATOR);
			let kitties_account_initial_balance = Balances::free_balance(&Kitties::account_id());

			assert_eq!(Kitties::next_kitty_id(), kitty_id_1);
			assert_ok!(Kitties::create(RuntimeOrigin::signed(VALID_KITTY_CREATOR), KITTY_A));

			assert_eq!(Kitties::next_kitty_id(), kitty_id_2);
			assert_ok!(Kitties::create(RuntimeOrigin::signed(VALID_KITTY_CREATOR), KITTY_B));

			assert_eq!(Kitties::next_kitty_id(), bred_kitty_id);
			assert_ok!(Kitties::breed(
				RuntimeOrigin::signed(VALID_KITTY_CREATOR),
				kitty_id_1,
				kitty_id_2,
				KITTY_C
			));

			assert_eq!(
				creator_initial_balance - KittyPrice::get() * 3,
				Balances::free_balance(&VALID_KITTY_CREATOR)
			);
			assert_eq!(
				kitties_account_initial_balance + KittyPrice::get() * 3,
				Balances::free_balance(&Kitties::account_id())
			);

			assert!(Kitties::kitties(bred_kitty_id).is_some());
			assert_eq!(Kitties::kitty_owner(bred_kitty_id), Some(VALID_KITTY_CREATOR));
			assert_eq!(Kitties::kitty_parents(bred_kitty_id), Some((kitty_id_1, kitty_id_2)));

			let bred_kitty = Kitties::kitties(bred_kitty_id).expect("Bred kitty was created");
			System::assert_last_event(
				Event::KittyBred {
					who: VALID_KITTY_CREATOR,
					kitty_id: bred_kitty_id,
					kitty: bred_kitty,
				}
				.into(),
			);
		});
	}

	mod failed_when {
		use super::*;

		#[test]
		fn bad_origin() {
			new_test_ext().execute_with(|| {
				let kitty_id = 0;
				assert_eq!(Kitties::next_kitty_id(), kitty_id);
				assert_ok!(Kitties::create(RuntimeOrigin::signed(VALID_KITTY_CREATOR), KITTY_A));
				assert_ok!(Kitties::create(RuntimeOrigin::signed(VALID_KITTY_CREATOR), KITTY_B));

				let bred_kitty_id = kitty_id + 2;
				assert_eq!(Kitties::next_kitty_id(), bred_kitty_id);

				assert_noop!(
					Kitties::breed(RuntimeOrigin::none(), kitty_id, kitty_id + 1, KITTY_C),
					BadOrigin
				);
				assert_noop!(
					Kitties::breed(RuntimeOrigin::root(), kitty_id, kitty_id + 1, KITTY_C),
					BadOrigin
				);
			});
		}

		#[test]
		fn parents_using_the_same_kitty_id() {
			new_test_ext().execute_with(|| {
				let kitty_id = 0;
				assert_eq!(Kitties::next_kitty_id(), kitty_id);
				assert_ok!(Kitties::create(RuntimeOrigin::signed(VALID_KITTY_CREATOR), KITTY_A));
				assert_ok!(Kitties::create(RuntimeOrigin::signed(VALID_KITTY_CREATOR), KITTY_B));

				let bred_kitty_id = kitty_id + 2;
				assert_eq!(Kitties::next_kitty_id(), bred_kitty_id);

				assert_noop!(
					Kitties::breed(
						RuntimeOrigin::signed(VALID_KITTY_CREATOR),
						kitty_id,
						kitty_id,
						KITTY_C
					),
					Error::<Test>::SameKittyId
				);
			});
		}

		#[test]
		fn parent_kitty_id_is_invalid() {
			new_test_ext().execute_with(|| {
				let kitty_id = 0;
				assert_eq!(Kitties::next_kitty_id(), kitty_id);
				assert_ok!(Kitties::create(RuntimeOrigin::signed(VALID_KITTY_CREATOR), KITTY_A));

				let bred_kitty_id = 1;
				assert_eq!(Kitties::next_kitty_id(), bred_kitty_id);

				assert_noop!(
					Kitties::breed(
						RuntimeOrigin::signed(VALID_KITTY_CREATOR),
						2,
						kitty_id,
						KITTY_C
					),
					Error::<Test>::InvalidKittyId
				);

				assert_noop!(
					Kitties::breed(
						RuntimeOrigin::signed(VALID_KITTY_CREATOR),
						kitty_id,
						2,
						KITTY_C
					),
					Error::<Test>::InvalidKittyId
				);
			});
		}

		#[test]
		fn next_kitty_id_overflow() {
			new_test_ext().execute_with(|| {
				let kitty_id_1 = 0;
				let kitty_id_2 = 1;

				assert_eq!(Kitties::next_kitty_id(), kitty_id_1);
				assert_ok!(Kitties::create(RuntimeOrigin::signed(VALID_KITTY_CREATOR), KITTY_A));

				assert_eq!(Kitties::next_kitty_id(), kitty_id_2);
				assert_ok!(Kitties::create(RuntimeOrigin::signed(VALID_KITTY_CREATOR), KITTY_B));

				crate::NextKittyId::<Test>::set(crate::KittyId::MAX);
				assert_noop!(
					Kitties::breed(
						RuntimeOrigin::signed(VALID_KITTY_CREATOR),
						kitty_id_1,
						kitty_id_2,
						KITTY_C
					),
					Error::<Test>::KittyIdCannotOverflow
				);
			});
		}

		#[test]
		fn not_enough_fund() {
			new_test_ext().execute_with(|| {
				let kitty_id_1 = 0;
				let kitty_id_2 = 1;
				let bred_kitty_id = 2;

				assert_eq!(Kitties::next_kitty_id(), kitty_id_1);
				assert_ok!(Kitties::create(RuntimeOrigin::signed(VALID_KITTY_CREATOR), KITTY_A));

				assert_eq!(Kitties::next_kitty_id(), kitty_id_2);
				assert_ok!(Kitties::create(RuntimeOrigin::signed(VALID_KITTY_CREATOR), KITTY_B));

				assert_eq!(Kitties::next_kitty_id(), bred_kitty_id);
				assert_noop!(
					Kitties::breed(
						RuntimeOrigin::signed(ACCOUNT_WITH_ONLY_EXISTENTIAL_DEPOSIT),
						kitty_id_1,
						kitty_id_2,
						KITTY_C
					),
					Token(FundsUnavailable)
				);
			});
		}

		#[test]
		fn not_able_to_keep_alive() {
			new_test_ext().execute_with(|| {
				let kitty_id_1 = 0;
				let kitty_id_2 = 1;
				let bred_kitty_id = 2;

				assert_eq!(Kitties::next_kitty_id(), kitty_id_1);
				assert_ok!(Kitties::create(RuntimeOrigin::signed(VALID_KITTY_CREATOR), KITTY_A));

				assert_eq!(Kitties::next_kitty_id(), kitty_id_2);
				assert_ok!(Kitties::create(RuntimeOrigin::signed(VALID_KITTY_CREATOR), KITTY_B));

				assert_eq!(Kitties::next_kitty_id(), bred_kitty_id);
				assert_noop!(
					Kitties::breed(
						RuntimeOrigin::signed(ACCOUNT_WITH_JUST_KITTY_PRICE_AMOUNT),
						kitty_id_1,
						kitty_id_2,
						KITTY_C
					),
					Token(NotExpendable)
				);
			});
		}
	}
}

mod transfer_kitty {
	use super::*;

	#[test]
	fn successful() {
		new_test_ext().execute_with(|| {
			let sender_initial_balance = Balances::free_balance(&VALID_KITTY_CREATOR);
			let receiver_initial_balance =
				Balances::free_balance(&ACCOUNT_WITH_ONLY_EXISTENTIAL_DEPOSIT);
			let kitties_account_initial_balance = Balances::free_balance(&Kitties::account_id());

			let kitty_id = 0;
			assert_eq!(Kitties::next_kitty_id(), kitty_id);
			assert_ok!(Kitties::create(RuntimeOrigin::signed(VALID_KITTY_CREATOR), KITTY_A));
			assert_eq!(Kitties::kitty_owner(kitty_id), Some(VALID_KITTY_CREATOR));

			assert_ok!(Kitties::transfer(
				RuntimeOrigin::signed(VALID_KITTY_CREATOR),
				ACCOUNT_WITH_ONLY_EXISTENTIAL_DEPOSIT,
				kitty_id
			));

			assert_eq!(
				sender_initial_balance - KittyPrice::get(),
				Balances::free_balance(&VALID_KITTY_CREATOR)
			);
			assert_eq!(
				kitties_account_initial_balance + KittyPrice::get(),
				Balances::free_balance(&Kitties::account_id())
			);
			assert_eq!(
				receiver_initial_balance,
				Balances::free_balance(&ACCOUNT_WITH_ONLY_EXISTENTIAL_DEPOSIT)
			);

			assert_eq!(Kitties::kitty_owner(kitty_id), Some(ACCOUNT_WITH_ONLY_EXISTENTIAL_DEPOSIT));
			System::assert_last_event(
				Event::KittyTransferred {
					who: VALID_KITTY_CREATOR,
					recipient: ACCOUNT_WITH_ONLY_EXISTENTIAL_DEPOSIT,
					kitty_id,
				}
				.into(),
			);

			assert_ok!(Kitties::transfer(
				RuntimeOrigin::signed(ACCOUNT_WITH_ONLY_EXISTENTIAL_DEPOSIT),
				VALID_KITTY_CREATOR,
				kitty_id
			));
			assert_eq!(Kitties::kitty_owner(kitty_id), Some(VALID_KITTY_CREATOR));
			System::assert_last_event(
				Event::KittyTransferred {
					who: ACCOUNT_WITH_ONLY_EXISTENTIAL_DEPOSIT,
					recipient: VALID_KITTY_CREATOR,
					kitty_id,
				}
				.into(),
			);
		});
	}

	mod failed_when {
		use super::*;

		#[test]
		fn bad_origin() {
			new_test_ext().execute_with(|| {
				let kitty_id = 0;
				assert_eq!(Kitties::next_kitty_id(), kitty_id);
				assert_ok!(Kitties::create(RuntimeOrigin::signed(VALID_KITTY_CREATOR), KITTY_A));
				assert_eq!(Kitties::kitty_owner(kitty_id), Some(VALID_KITTY_CREATOR));

				assert_noop!(
					Kitties::transfer(
						RuntimeOrigin::none(),
						ACCOUNT_WITH_ONLY_EXISTENTIAL_DEPOSIT,
						kitty_id
					),
					BadOrigin
				);
				assert_noop!(
					Kitties::transfer(
						RuntimeOrigin::root(),
						ACCOUNT_WITH_ONLY_EXISTENTIAL_DEPOSIT,
						kitty_id
					),
					BadOrigin
				);
			});
		}

		#[test]
		fn invalid_kitty_id() {
			new_test_ext().execute_with(|| {
				let kitty_id = 0;
				let invalid_kitty_id = 100;

				assert_eq!(Kitties::next_kitty_id(), kitty_id);
				assert_ok!(Kitties::create(RuntimeOrigin::signed(VALID_KITTY_CREATOR), KITTY_A));
				assert_eq!(Kitties::kitty_owner(kitty_id), Some(VALID_KITTY_CREATOR));

				assert_noop!(
					Kitties::transfer(
						RuntimeOrigin::signed(VALID_KITTY_CREATOR),
						ACCOUNT_WITH_ONLY_EXISTENTIAL_DEPOSIT,
						invalid_kitty_id
					),
					Error::<Test>::InvalidKittyId
				);
			});
		}

		#[test]
		fn sender_was_not_kitty_owner() {
			new_test_ext().execute_with(|| {
				let kitty_id = 0;
				assert_eq!(Kitties::next_kitty_id(), kitty_id);
				assert_ok!(Kitties::create(RuntimeOrigin::signed(VALID_KITTY_CREATOR), KITTY_A));
				assert_eq!(Kitties::kitty_owner(kitty_id), Some(VALID_KITTY_CREATOR));

				assert_noop!(
					Kitties::transfer(
						RuntimeOrigin::signed(ACCOUNT_WITH_ONLY_EXISTENTIAL_DEPOSIT),
						VALID_KITTY_CREATOR,
						kitty_id
					),
					Error::<Test>::NotOwner
				);
			});
		}
	}
}

mod put_kitty_on_sale {
	use super::*;

	#[test]
	fn successful() {
		new_test_ext().execute_with(|| {
			let kitty_id = 0;
			assert_eq!(Kitties::next_kitty_id(), kitty_id);
			let _ = Kitties::create(RuntimeOrigin::signed(VALID_KITTY_CREATOR), KITTY_A);

			assert!(Kitties::kitty_on_sale(kitty_id).is_none());
			let creator_initial_balance = Balances::free_balance(&VALID_KITTY_CREATOR);

			assert_ok!(Kitties::sale(RuntimeOrigin::signed(VALID_KITTY_CREATOR), kitty_id));

			assert!(Kitties::kitty_on_sale(kitty_id).is_some());
			System::assert_last_event(
				Event::KittyOnSale { who: VALID_KITTY_CREATOR, kitty_id }.into(),
			);
			assert_eq!(creator_initial_balance, Balances::free_balance(&VALID_KITTY_CREATOR));
		});
	}

	mod failed_when {
		use super::*;

		#[test]
		fn bad_origin() {
			new_test_ext().execute_with(|| {
				let kitty_id = 0;
				assert_eq!(Kitties::next_kitty_id(), kitty_id);
				let _ = Kitties::create(RuntimeOrigin::signed(VALID_KITTY_CREATOR), KITTY_A);

				assert_noop!(Kitties::sale(RuntimeOrigin::none(), kitty_id), BadOrigin);
				assert_noop!(Kitties::sale(RuntimeOrigin::root(), kitty_id), BadOrigin);
			});
		}

		#[test]
		fn invalid_kitty_id() {
			new_test_ext().execute_with(|| {
				let kitty_id = 0;
				let invalid_kitty_id = 100;

				assert_eq!(Kitties::next_kitty_id(), kitty_id);
				let _ = Kitties::create(RuntimeOrigin::signed(VALID_KITTY_CREATOR), KITTY_A);

				assert_noop!(
					Kitties::sale(RuntimeOrigin::signed(VALID_KITTY_CREATOR), invalid_kitty_id),
					Error::<Test>::InvalidKittyId
				);
			});
		}

		#[test]
		fn caller_was_not_kitty_owner() {
			new_test_ext().execute_with(|| {
				let kitty_id = 0;
				assert_eq!(Kitties::next_kitty_id(), kitty_id);
				assert_ok!(Kitties::create(RuntimeOrigin::signed(VALID_KITTY_CREATOR), KITTY_A));
				assert_eq!(Kitties::kitty_owner(kitty_id), Some(VALID_KITTY_CREATOR));

				assert_noop!(
					Kitties::sale(
						RuntimeOrigin::signed(ACCOUNT_WITH_ONLY_EXISTENTIAL_DEPOSIT),
						kitty_id
					),
					Error::<Test>::NotOwner
				);
			});
		}

		#[test]
		fn alreay_on_sale() {
			new_test_ext().execute_with(|| {
				let kitty_id = 0;
				assert_eq!(Kitties::next_kitty_id(), kitty_id);
				let _ = Kitties::create(RuntimeOrigin::signed(VALID_KITTY_CREATOR), KITTY_A);

				assert_ok!(Kitties::sale(RuntimeOrigin::signed(VALID_KITTY_CREATOR), kitty_id));

				assert_noop!(
					Kitties::sale(RuntimeOrigin::signed(VALID_KITTY_CREATOR), kitty_id),
					Error::<Test>::AlreadyOnSale
				);
			});
		}
	}
}

mod buy_kitty {
	use super::*;

	#[test]
	fn successful() {
		new_test_ext().execute_with(|| {
			let kitty_id = 0;

			assert_eq!(Kitties::next_kitty_id(), kitty_id);
			let _ = Kitties::create(RuntimeOrigin::signed(VALID_KITTY_CREATOR), KITTY_A);

			assert_eq!(Kitties::kitty_owner(kitty_id), Some(VALID_KITTY_CREATOR));
			let _ = Kitties::sale(RuntimeOrigin::signed(VALID_KITTY_CREATOR), kitty_id);

			let seller_initial_balance = Balances::free_balance(&VALID_KITTY_CREATOR);
			let buyer_initial_balance = Balances::free_balance(&VALID_KITTY_BUYER);
			let kitties_account_initial_balance = Balances::free_balance(&Kitties::account_id());

			assert_ok!(Kitties::buy(RuntimeOrigin::signed(VALID_KITTY_BUYER), kitty_id));

			assert_eq!(Kitties::kitty_owner(kitty_id), Some(VALID_KITTY_BUYER));
			System::assert_last_event(
				Event::KittyBought { who: VALID_KITTY_BUYER, kitty_id }.into(),
			);

			assert_eq!(
				seller_initial_balance + KittyPrice::get(),
				Balances::free_balance(&VALID_KITTY_CREATOR)
			);
			assert_eq!(
				buyer_initial_balance - KittyPrice::get(),
				Balances::free_balance(&VALID_KITTY_BUYER)
			);
			assert_eq!(
				kitties_account_initial_balance,
				Balances::free_balance(&Kitties::account_id())
			);
		});
	}

	mod failed_when {
		use crate::KittyOwner;

		use super::*;

		#[test]
		fn bad_origin() {
			new_test_ext().execute_with(|| {
				let kitty_id = 0;

				assert_eq!(Kitties::next_kitty_id(), kitty_id);
				let _ = Kitties::create(RuntimeOrigin::signed(VALID_KITTY_CREATOR), KITTY_A);

				assert_eq!(Kitties::kitty_owner(kitty_id), Some(VALID_KITTY_CREATOR));
				let _ = Kitties::sale(RuntimeOrigin::signed(VALID_KITTY_CREATOR), kitty_id);

				assert_noop!(Kitties::buy(RuntimeOrigin::none(), kitty_id), BadOrigin);
				assert_noop!(Kitties::buy(RuntimeOrigin::root(), kitty_id), BadOrigin);
			});
		}

		#[test]
		fn invalid_kitty_id() {
			new_test_ext().execute_with(|| {
				let kitty_id = 0;
				let invalid_kitty_id = 100;

				assert_eq!(Kitties::next_kitty_id(), kitty_id);
				let _ = Kitties::create(RuntimeOrigin::signed(VALID_KITTY_CREATOR), KITTY_A);

				assert_eq!(Kitties::kitty_owner(kitty_id), Some(VALID_KITTY_CREATOR));
				let _ = Kitties::sale(RuntimeOrigin::signed(VALID_KITTY_CREATOR), kitty_id);

				assert_noop!(
					Kitties::buy(RuntimeOrigin::signed(VALID_KITTY_BUYER), invalid_kitty_id),
					Error::<Test>::InvalidKittyId
				);
			});
		}

		#[test]
		fn kitty_has_no_owner() {
			new_test_ext().execute_with(|| {
				let kitty_id = 0;

				assert_eq!(Kitties::next_kitty_id(), kitty_id);
				let _ = Kitties::create(RuntimeOrigin::signed(VALID_KITTY_CREATOR), KITTY_A);

				assert_eq!(Kitties::kitty_owner(kitty_id), Some(VALID_KITTY_CREATOR));
				let _ = Kitties::sale(RuntimeOrigin::signed(VALID_KITTY_CREATOR), kitty_id);

				<KittyOwner<Test>>::remove(kitty_id);
				assert_noop!(
					Kitties::buy(RuntimeOrigin::signed(VALID_KITTY_BUYER), kitty_id),
					Error::<Test>::NoOwner
				);
			});
		}

		#[test]
		fn buyer_already_owned() {
			new_test_ext().execute_with(|| {
				let kitty_id = 0;

				assert_eq!(Kitties::next_kitty_id(), kitty_id);
				let _ = Kitties::create(RuntimeOrigin::signed(VALID_KITTY_CREATOR), KITTY_A);

				assert_eq!(Kitties::kitty_owner(kitty_id), Some(VALID_KITTY_CREATOR));
				let _ = Kitties::sale(RuntimeOrigin::signed(VALID_KITTY_CREATOR), kitty_id);

				assert_noop!(
					Kitties::buy(RuntimeOrigin::signed(VALID_KITTY_CREATOR), kitty_id),
					Error::<Test>::AlreadyOwned
				);
			});
		}

		#[test]
		fn kitty_was_not_on_sale() {
			new_test_ext().execute_with(|| {
				let kitty_id = 0;

				assert_eq!(Kitties::next_kitty_id(), kitty_id);
				let _ = Kitties::create(RuntimeOrigin::signed(VALID_KITTY_CREATOR), KITTY_A);

				assert_noop!(
					Kitties::buy(RuntimeOrigin::signed(VALID_KITTY_BUYER), kitty_id),
					Error::<Test>::NotOnSale
				);
			});
		}

		#[test]
		fn buyer_has_not_enough_fund() {
			new_test_ext().execute_with(|| {
				let kitty_id = 0;

				assert_eq!(Kitties::next_kitty_id(), kitty_id);
				let _ = Kitties::create(RuntimeOrigin::signed(VALID_KITTY_CREATOR), KITTY_A);

				assert_eq!(Kitties::kitty_owner(kitty_id), Some(VALID_KITTY_CREATOR));
				let _ = Kitties::sale(RuntimeOrigin::signed(VALID_KITTY_CREATOR), kitty_id);

				assert_noop!(
					Kitties::buy(
						RuntimeOrigin::signed(ACCOUNT_WITH_ONLY_EXISTENTIAL_DEPOSIT),
						kitty_id
					),
					Token(FundsUnavailable)
				);
			});
		}

		#[test]
		fn buyer_cannot_keep_alive() {
			new_test_ext().execute_with(|| {
				let kitty_id = 0;

				assert_eq!(Kitties::next_kitty_id(), kitty_id);
				let _ = Kitties::create(RuntimeOrigin::signed(VALID_KITTY_CREATOR), KITTY_A);

				assert_eq!(Kitties::kitty_owner(kitty_id), Some(VALID_KITTY_CREATOR));
				let _ = Kitties::sale(RuntimeOrigin::signed(VALID_KITTY_CREATOR), kitty_id);

				assert_noop!(
					Kitties::buy(
						RuntimeOrigin::signed(ACCOUNT_WITH_JUST_KITTY_PRICE_AMOUNT),
						kitty_id
					),
					Token(NotExpendable)
				);
			});
		}
	}
}
