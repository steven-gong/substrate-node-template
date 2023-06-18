#![cfg_attr(not(feature = "std"), no_std)]

/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// <https://docs.substrate.io/reference/frame-pallets/>
pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;
pub mod weights;
pub use weights::*;

use serde::{Deserialize, Deserializer};
use sp_runtime::KeyTypeId;

/// Defines application identifier for crypto keys of this module.
///
/// Every module that deals with signatures needs to declare its unique identifier for
/// its crypto keys.
/// When offchain worker is signing transactions it's going to request keys of type
/// `KeyTypeId` from the keystore and use the ones it finds to sign the transaction.
/// The keys can be inserted manually via RPC (see `author_insertKey`).
pub const KEY_TYPE: KeyTypeId = KeyTypeId(*b"btc!");

/// Based on the above `KeyTypeId` we need to generate a pallet-specific crypto type wrappers.
/// We can use from supported crypto kinds (`sr25519`, `ed25519` and `ecdsa`) and augment
/// the types with this pallet-specific identifier.
pub mod crypto {
	use super::KEY_TYPE;
	use sp_core::sr25519::Signature as Sr25519Signature;
	use sp_runtime::{
		app_crypto::{app_crypto, sr25519},
		traits::Verify,
		MultiSignature, MultiSigner,
	};
	app_crypto!(sr25519, KEY_TYPE);

	pub struct TestAuthId;

	impl frame_system::offchain::AppCrypto<MultiSigner, MultiSignature> for TestAuthId {
		type RuntimeAppPublic = Public;
		type GenericSignature = sp_core::sr25519::Signature;
		type GenericPublic = sp_core::sr25519::Public;
	}

	// implemented for mock runtime in test
	impl frame_system::offchain::AppCrypto<<Sr25519Signature as Verify>::Signer, Sr25519Signature>
		for TestAuthId
	{
		type RuntimeAppPublic = Public;
		type GenericSignature = sp_core::sr25519::Signature;
		type GenericPublic = sp_core::sr25519::Public;
	}
}

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use core::convert::TryInto;
	use frame_support::{inherent::Vec, sp_io::offchain_index, pallet_prelude::*, traits::Get};
	use frame_system::{offchain::{AppCrypto, CreateSignedTransaction, SendUnsignedTransaction, Signer, SignedPayload, SigningTypes}, pallet_prelude::*};
	use sp_runtime::offchain::{http, Duration};
	use sp_std::{prelude::*, str};
	use lite_json::json::JsonValue;

	const ONCHAIN_TX_KEY: &[u8] = b"template_pallet::indexing1";
	const CRYPTOCOMPARE_MIN_API_URL: &str  = "https://min-api.cryptocompare.com/data/price?fsym=BTC&tsyms=USD";

	#[derive(Debug, serde::Deserialize, Encode, Decode, Default)]
	struct IndexingData{
		#[serde(deserialize_with = "de_string_to_bytes")]
		name: Vec<u8>, 
		value: u32
	}

    pub fn de_string_to_bytes<'de, D>(de: D) -> Result<Vec<u8>, D::Error>
        where
        D: Deserializer<'de>,
        {
            let s: &str = Deserialize::deserialize(de)?;
            Ok(s.as_bytes().to_vec())
        }

	#[pallet::pallet]
	pub struct Pallet<T>(_);

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config + CreateSignedTransaction<Call<Self>> {
		/// The identifier type for an offchain worker.
		type AuthorityId: AppCrypto<Self::Public, Self::Signature>;
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
		/// Type representing the weight of this pallet
		type WeightInfo: WeightInfo;
		/// Maximum number of prices.
		#[pallet::constant]
		type MaxPrices: Get<u32>;
		/// Number of blocks of cooldown after unsigned transaction is included.
		///
		/// This ensures that we only accept unsigned transactions once, every `UnsignedInterval`
		/// blocks.
		#[pallet::constant]
		type UnsignedInterval: Get<Self::BlockNumber>;
		/// A configuration for base priority of unsigned transactions.
		///
		/// This is exposed so that it can be tuned for particular runtime, when
		/// multiple pallets send unsigned transactions.
		#[pallet::constant]
		type UnsignedPriority: Get<TransactionPriority>;
	}	

	/// A vector of recently submitted prices.
	///
	/// This is used to calculate average price, should have bounded size.
	#[pallet::storage]
	#[pallet::getter(fn prices)]
	pub(super) type Prices<T: Config> = StorageValue<_, BoundedVec<u32, T::MaxPrices>, ValueQuery>;

	/// Defines the block when next unsigned transaction will be accepted.
	///
	/// To prevent spam of unsigned (and unpaid!) transactions on the network,
	/// we only allow one transaction every `T::UnsignedInterval` blocks.
	/// This storage entry defines when new transaction is going to be accepted.
	#[pallet::storage]
	#[pallet::getter(fn next_unsigned_at)]
	pub(super) type NextUnsignedAt<T: Config> = StorageValue<_, T::BlockNumber, ValueQuery>;

	// The pallet's runtime storage items.
	// https://docs.substrate.io/main-docs/build/runtime-storage/
	#[pallet::storage]
	#[pallet::getter(fn something)]
	// Learn more about declaring storage items:
	// https://docs.substrate.io/main-docs/build/runtime-storage/#declaring-storage-items
	pub type Something<T> = StorageValue<_, u32>;

	/// Payload used by this example crate to hold price
	/// data required to submit a transaction.
	#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, scale_info::TypeInfo)]
	pub struct PricePayload<Public, BlockNumber> {
		block_number: BlockNumber,
		price: u32,
		public: Public,
	}

	impl<T: SigningTypes> SignedPayload<T> for PricePayload<T::Public, T::BlockNumber> {
		fn public(&self) -> T::Public {
			self.public.clone()
		}
	}


	// Pallets use events to inform users when important changes are made.
	// https://docs.substrate.io/main-docs/build/events-errors/
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Event documentation should end with an array that provides descriptive names for event
		/// parameters. [something, who]
		SomethingStored {
			something: u32,
			who: T::AccountId,
		},
		/// Event generated when a value is stored in local storage by an offchain worker
		OffchainStored {
			key: Vec<u8>,
			number: u32,
			who: T::AccountId,
		},
		/// Event generated when new price is accepted to contribute to the average.
		NewPrice { 
			price: u32, 
			maybe_who: Option<T::AccountId> 
		},
	}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		/// Error names should be descriptive.
		NoneValue,
		/// Errors should have helpful documentation associated with them.
		StorageOverflow,
	}

	#[pallet::validate_unsigned]
	impl<T: Config> ValidateUnsigned for Pallet<T> {
		type Call = Call<T>;

		/// Validate unsigned call to this module.
		///
		/// By default unsigned transactions are disallowed, but implementing the validator
		/// here we make sure that some particular calls (the ones produced by offchain worker)
		/// are being whitelisted and marked as valid.
		fn validate_unsigned(_source: TransactionSource, call: &Self::Call) -> TransactionValidity {
			// Firstly let's check that we call the right function.
			if let Call::submit_price_unsigned_with_signed_payload {
				price_payload: ref payload,
				ref signature,
			} = call
			{
				let signature_valid =
					SignedPayload::<T>::verify::<T::AuthorityId>(payload, signature.clone());
				if !signature_valid {
					return InvalidTransaction::BadProof.into()
				}
				Self::validate_transaction_parameters(&payload.block_number, &payload.price)
			} else {
				InvalidTransaction::Call.into()
			}
		}
	}

	// Dispatchable functions allows users to interact with the pallet and invoke state changes.
	// These functions materialize as "extrinsics", which are often compared to transactions.
	// Dispatchable functions must be annotated with a weight and must return a DispatchResult.
	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// An example dispatchable that takes a singles value as a parameter, writes the value to
		/// storage and emits an event. This function must be dispatched by a signed extrinsic.
		#[pallet::call_index(0)]
		#[pallet::weight(T::WeightInfo::do_something())]
		pub fn do_something(origin: OriginFor<T>, something: u32) -> DispatchResult {
			// Check that the extrinsic was signed and get the signer.
			// This function will return an error if the extrinsic is not signed.
			// https://docs.substrate.io/main-docs/build/origins/
			let who = ensure_signed(origin)?;

			// Update storage.
			<Something<T>>::put(something);

			// Emit an event.
			Self::deposit_event(Event::SomethingStored { something, who });
			// Return a successful DispatchResultWithPostInfo
			Ok(())
		}

		/// An example dispatchable that may throw a custom error.
		#[pallet::call_index(1)]
		#[pallet::weight(T::WeightInfo::cause_error())]
		pub fn cause_error(origin: OriginFor<T>) -> DispatchResult {
			let _who = ensure_signed(origin)?;

			// Read a value from storage.
			match <Something<T>>::get() {
				// Return an error if the value has not been set.
				None => return Err(Error::<T>::NoneValue.into()),
				Some(old) => {
					// Increment the value read from storage; will error in the event of overflow.
					let new = old.checked_add(1).ok_or(Error::<T>::StorageOverflow)?;
					// Update the value in storage with the incremented result.
					<Something<T>>::put(new);
					Ok(())
				},
			}
		}

		#[pallet::call_index(2)]
		#[pallet::weight({0})]
		pub fn write_offchain_storage(origin: OriginFor<T>, number: u32) -> DispatchResult {
			let who = ensure_signed(origin)?;

			let key = ONCHAIN_TX_KEY.to_vec();
			let data = IndexingData{
				name: b"submit_number_unsigned".to_vec(), 
				value: number
			};
			offchain_index::set(&key, &data.encode());

			// Emit an event.
			Self::deposit_event(Event::OffchainStored { key, number, who });

			Ok(())
		}

		#[pallet::call_index(3)]
		#[pallet::weight({4})]
		pub fn submit_price_unsigned_with_signed_payload(
			origin: OriginFor<T>, 
			price_payload: PricePayload<T::Public, T::BlockNumber>,
			_signature: T::Signature,
		) -> DispatchResult {
			// This ensures that the function can only be called via unsigned transaction.
			ensure_none(origin)?;
			
			// TODO: Verify signature is signed by a oracle member account

			// Add the price to the on-chain list, but mark it as coming from an empty address.
			Self::add_price(None, price_payload.price);

			// now increment the block number at which we expect next unsigned transaction.
			let current_block = <frame_system::Pallet<T>>::block_number();
			<NextUnsignedAt<T>>::put(current_block + T::UnsignedInterval::get());

			Ok(())
		}
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
		fn offchain_worker(block_number: T::BlockNumber) {
			log::info!("OCW ==> Hello World from offchain workers {:?}", block_number);

			let parent_hash = <frame_system::Pallet<T>>::block_hash(block_number - 1u32.into());
			log::debug!("Current block: {:?} (parent hash: {:?})", block_number, parent_hash);

			let res = Self::fetch_price_and_send_unsigned_for_any_account(block_number);
			if let Err(e) = res {
				log::error!("Error: {}", e);
			}
		}
	}

	impl<T: Config> Pallet<T> {
		/// A helper function to fetch the price, sign payload and send an unsigned transaction
		fn fetch_price_and_send_unsigned_for_any_account(
			block_number: T::BlockNumber,
		) -> Result<(), &'static str> {
			// Make sure we don't fetch the price if unsigned transaction is going to be rejected
			// anyway.
			let next_unsigned_at = <NextUnsignedAt<T>>::get();
			if next_unsigned_at > block_number {
				return Err("Too early to send unsigned transaction")
			}

			// Make an external HTTP request to fetch the current price.
			// Note this call will block until response is received.
			let price = Self::fetch_price().map_err(|_| "Failed to fetch price")?;

			// -- Sign using any account
			let (_, result) = Signer::<T, T::AuthorityId>::any_account()
				.send_unsigned_transaction(
					|account| PricePayload { price, block_number, public: account.public.clone() },
					|payload, signature| Call::submit_price_unsigned_with_signed_payload {
						price_payload: payload,
						signature,
					},
				)
				.ok_or("No local accounts available.")?;
			result.map_err(|()| "Unable to submit transaction")?;

			Ok(())
		}

		/// Fetch current price and return the result in cents.
		fn fetch_price() -> Result<u32, http::Error> {
			// We want to keep the offchain worker execution time reasonable, so we set a hard-coded
			// deadline to 2s to complete the external call.
			// You can also wait indefinitely for the response, however you may still get a timeout
			// coming from the host machine.
			let deadline = sp_io::offchain::timestamp().add(Duration::from_millis(2_000));
			// Initiate an external HTTP GET request.
			// This is using high-level wrappers from `sp_runtime`, for the low-level calls that
			// you can find in `sp_io`. The API is trying to be similar to `request`, but
			// since we are running in a custom WASM execution environment we can't simply
			// import the library here.
			let request = http::Request::get(CRYPTOCOMPARE_MIN_API_URL);
			// We set the deadline for sending of the request, note that awaiting response can
			// have a separate deadline. Next we send the request, before that it's also possible
			// to alter request headers or stream body content in case of non-GET requests.
			let pending = request.deadline(deadline).send().map_err(|_| http::Error::IoError)?;

			// The request is already being processed by the host, we are free to do anything
			// else in the worker (we can send multiple concurrent requests too).
			// At some point however we probably want to check the response though,
			// so we can block current thread and wait for it to finish.
			// Note that since the request is being driven by the host, we don't have to wait
			// for the request to have it complete, we will just not read the response.
			let response = pending.try_wait(deadline).map_err(|_| http::Error::DeadlineReached)??;

			// Let's check the status code before we proceed to reading the response.
			if response.code != 200 {
				log::warn!("OCW ==> Unexpected status code: {}", response.code);
				return Err(http::Error::Unknown)
			}

			// Next we want to fully read the response body and collect it to a vector of bytes.
			// Note that the return object allows you to read the body in chunks as well
			// with a way to control the deadline.
			let body = response.body().collect::<Vec<u8>>();

			// Create a str slice from the body.
			let body_str = sp_std::str::from_utf8(&body).map_err(|_| {
				log::warn!("OCW ==> No UTF8 body");
				http::Error::Unknown
			})?;

			let price = match Self::parse_price(body_str) {
				Some(price) => Ok(price),
				None => {
					log::warn!("OCW ==> Unable to extract price from the response: {:?}", body_str);
					Err(http::Error::Unknown)
				},
			}?;

			log::warn!("OCW ==> Got price: {} cents", price);

			Ok(price)
		}

		/// Add new price to the list.
		fn add_price(maybe_who: Option<T::AccountId>, price: u32) {
			log::info!("OCW ==> Adding to the average: {}", price);
			<Prices<T>>::mutate(|prices| {
				if prices.try_push(price).is_err() {
					// handles prices index overflow
					prices[(price % T::MaxPrices::get()) as usize] = price;
				}
			});

			let average = Self::average_price()
				.expect("The average is not empty, because it was just mutated; qed");
			log::info!("OCW ==> Current average price is: {}", average);
			// here we are raising the NewPrice event
			Self::deposit_event(Event::NewPrice { price, maybe_who });
		}

		/// Calculate current average price.
		fn average_price() -> Option<u32> {
			let prices = <Prices<T>>::get();
			if prices.is_empty() {
				None
			} else {
				Some(prices.iter().fold(0_u32, |a, b| a.saturating_add(*b)) / prices.len() as u32)
			}
		}

		/// Parse the price from the given JSON string using `lite-json`.
		///
		/// Returns `None` when parsing failed or `Some(price in cents)` when parsing is successful.
		fn parse_price(price_str: &str) -> Option<u32> {
			let val = lite_json::parse_json(price_str);
			let price = match val.ok()? {
				JsonValue::Object(obj) => {
					let (_, v) = obj.into_iter().find(|(k, _)| k.iter().copied().eq("USD".chars()))?;
					match v {
						JsonValue::Number(number) => number,
						_ => return None,
					}
				},
				_ => return None,
			};

			let exp = price.fraction_length.saturating_sub(2);
			Some(price.integer as u32 * 100 + (price.fraction / 10_u64.pow(exp)) as u32)
		}

		fn validate_transaction_parameters(
			block_number: &T::BlockNumber,
			new_price: &u32,
		) -> TransactionValidity {
			// Now let's check if the transaction has any chance to succeed.
			let next_unsigned_at = <NextUnsignedAt<T>>::get();
			if &next_unsigned_at > block_number {
				return InvalidTransaction::Stale.into()
			}
			// Let's make sure to reject transactions from the future.
			let current_block = <frame_system::Pallet<T>>::block_number();
			if &current_block < block_number {
				return InvalidTransaction::Future.into()
			}
	
			// We prioritize transactions that are more far away from current average.
			//
			// Note this doesn't make much sense when building an actual oracle, but this example
			// is here mostly to show off offchain workers capabilities, not about building an
			// oracle.
			let avg_price = Self::average_price()
				.map(|price| if &price > new_price { price - new_price } else { new_price - price })
				.unwrap_or(0);
	
			ValidTransaction::with_tag_prefix("ExampleOffchainWorker")
				// We set base priority to 2**20 and hope it's included before any other
				// transactions in the pool. Next we tweak the priority depending on how much
				// it differs from the current average. (the more it differs the more priority it
				// has).
				.priority(T::UnsignedPriority::get().saturating_add(avg_price as _))
				// This transaction does not require anything else to go before into the pool.
				// In theory we could require `previous_unsigned_at` transaction to go first,
				// but it's not necessary in our case.
				//.and_requires()
				// We set the `provides` tag to be the same as `next_unsigned_at`. This makes
				// sure only one transaction produced after `next_unsigned_at` will ever
				// get to the transaction pool and will end up in the block.
				// We can still have multiple transactions compete for the same "spot",
				// and the one with higher priority will replace other one in the pool.
				.and_provides(next_unsigned_at)
				// The transaction is only valid for next 5 blocks. After that it's
				// going to be revalidated by the pool.
				.longevity(5)
				// It's fine to propagate that transaction to other peers, which means it can be
				// created even by nodes that don't produce blocks.
				// Note that sometimes it's better to keep it for yourself (if you are the block
				// producer), since for instance in some schemes others may copy your solution and
				// claim a reward.
				.propagate(true)
				.build()
		}
	}
}