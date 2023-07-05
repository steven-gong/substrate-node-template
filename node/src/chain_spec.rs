use hex_literal::hex;
use node_template_runtime::{
	AccountId, BabeConfig, BalancesConfig, GenesisConfig, GrandpaConfig, Signature, SudoConfig,
	SystemConfig, WASM_BINARY, BABE_GENESIS_EPOCH_CONFIG, SessionConfig, StakingConfig,ImOnlineConfig,
	Perbill, Balance, constants::currency::DOLLARS, MaxNominations, StakerStatus, SessionKeys
};
use sc_service::ChainType;
use sp_consensus_babe::AuthorityId as BabeId;
use sp_consensus_grandpa::AuthorityId as GrandpaId;
use pallet_im_online::sr25519::AuthorityId as ImOnlineId;
use sp_core::{crypto::UncheckedInto, sr25519, Pair, Public};
use sc_telemetry::TelemetryEndpoints;
use sp_runtime::traits::{IdentifyAccount, Verify};

// The URL for the telemetry server.
const STAGING_TELEMETRY_URL: &str = "wss://telemetry.polkadot.io/submit/";

/// Specialized `ChainSpec`. This is a specialization of the general Substrate ChainSpec type.
pub type ChainSpec = sc_service::GenericChainSpec<GenesisConfig>;

/// Generate a crypto pair from seed.
pub fn get_from_seed<TPublic: Public>(seed: &str) -> <TPublic::Pair as Pair>::Public {
	TPublic::Pair::from_string(&format!("//{}", seed), None)
		.expect("static values are valid; qed")
		.public()
}

type AccountPublic = <Signature as Verify>::Signer;

/// Generate an account ID from seed.
pub fn get_account_id_from_seed<TPublic: Public>(seed: &str) -> AccountId
where
	AccountPublic: From<<TPublic::Pair as Pair>::Public>,
{
	AccountPublic::from(get_from_seed::<TPublic>(seed)).into_account()
}

/// Generate an Babe authority key.
pub fn authority_keys_from_seed(s: &str) -> (AccountId, AccountId, BabeId, GrandpaId, ImOnlineId) {
	(
		get_account_id_from_seed::<sr25519::Public>(&format!("{}//stash", s)),
		get_account_id_from_seed::<sr25519::Public>(s),
		get_from_seed::<BabeId>(s),
		get_from_seed::<GrandpaId>(s),
		get_from_seed::<ImOnlineId>(s),
	)
}

fn session_keys(
	babe: BabeId,
	grandpa: GrandpaId,
	im_online: ImOnlineId,
) -> SessionKeys {
	SessionKeys { babe, grandpa, im_online }
}

pub fn development_config() -> Result<ChainSpec, String> {
	let wasm_binary = WASM_BINARY.ok_or_else(|| "Development wasm not available".to_string())?;

	Ok(ChainSpec::from_genesis(
		// Name
		"Development",
		// ID
		"dev",
		ChainType::Development,
		move || {
			testnet_genesis(
				wasm_binary,
				// Initial PoA authorities
				vec![authority_keys_from_seed("Alice")],
				// Initial nominators
				vec![],
				// Sudo account
				get_account_id_from_seed::<sr25519::Public>("Alice"),
				// Pre-funded accounts
				vec![
					get_account_id_from_seed::<sr25519::Public>("Alice"),
					get_account_id_from_seed::<sr25519::Public>("Bob"),
					get_account_id_from_seed::<sr25519::Public>("Alice//stash"),
					get_account_id_from_seed::<sr25519::Public>("Bob//stash"),
				],
				true,
			)
		},
		// Bootnodes
		vec![],
		// Telemetry
		None,
		// Protocol ID
		None,
		None,
		// Properties
		None,
		// Extensions
		None,
	))
}

pub fn local_testnet_config() -> Result<ChainSpec, String> {
	let wasm_binary = WASM_BINARY.ok_or_else(|| "Development wasm not available".to_string())?;

	Ok(ChainSpec::from_genesis(
		// Name
		"Local Testnet",
		// ID
		"local_testnet",
		ChainType::Local,
		move || {
			testnet_genesis(
				wasm_binary,
				// Initial PoA authorities
				vec![authority_keys_from_seed("Alice"), authority_keys_from_seed("Bob")],
				// Initial nominators
				vec![],
				// Sudo account
				get_account_id_from_seed::<sr25519::Public>("Alice"),
				// Pre-funded accounts
				vec![
					get_account_id_from_seed::<sr25519::Public>("Alice"),
					get_account_id_from_seed::<sr25519::Public>("Bob"),
					get_account_id_from_seed::<sr25519::Public>("Charlie"),
					get_account_id_from_seed::<sr25519::Public>("Dave"),
					get_account_id_from_seed::<sr25519::Public>("Eve"),
					get_account_id_from_seed::<sr25519::Public>("Ferdie"),
					get_account_id_from_seed::<sr25519::Public>("Alice//stash"),
					get_account_id_from_seed::<sr25519::Public>("Bob//stash"),
					get_account_id_from_seed::<sr25519::Public>("Charlie//stash"),
					get_account_id_from_seed::<sr25519::Public>("Dave//stash"),
					get_account_id_from_seed::<sr25519::Public>("Eve//stash"),
					get_account_id_from_seed::<sr25519::Public>("Ferdie//stash"),
				],
				true,
			)
		},
		// Bootnodes
		vec![],
		// Telemetry
		None,
		// Protocol ID
		None,
		// Properties
		None,
		None,
		// Extensions
		None,
	))
}

pub fn staging_network_config() -> ChainSpec {
	let boot_nodes = vec![];

	ChainSpec::from_genesis(
		"Staging Network",
		"staging_network",
		ChainType::Live,
		staging_network_config_genesis,
		boot_nodes,
		Some(
			TelemetryEndpoints::new(vec![(STAGING_TELEMETRY_URL.to_string(), 0)]).expect("Staging telemetry url is valid; qed"),
		),
		None,
		None,
		None,
		Default::default(),
	)
}

fn staging_network_config_genesis() -> GenesisConfig {
	let wasm_binary = WASM_BINARY.expect(
		"Development wasm binary is not available. This means the client is built with \
		 `SKIP_WASM_BUILD` flag and it is only usable for production chains. Please rebuild with \
		 the flag disabled.",
	);

	// for i in 1 2 3 4; do for j in stash controller; do ./target/release/node-template key inspect "$SECRET//$i//$j"; done; done
	// for i in 1 2 3 4; do for j in babe; do ./target/release/node-template key inspect --scheme sr25519 "$SECRET//$i//$j"; done; done
	// for i in 1 2 3 4; do for j in grandpa; do ./target/release/node-template key inspect --scheme ed25519 "$SECRET//$i//$j"; done; done
	// for i in 1 2 3 4; do for j in im_online; do ./target/release/node-template key inspect --scheme sr25519 "$SECRET//$i//$j"; done; done
	let initial_authorities: Vec<(AccountId, AccountId, BabeId, GrandpaId, ImOnlineId)> = vec![
		(
			// 5F4VYJHcjET8wHv5aNgF144mL8oMYQ9KC4C97S7sXMhoQSaS
			hex!["848b3d134dc0d666b786fa5864d26e21c556220dbca2da504ed59fe1e6300e4f"].into(),
			// 5H8vPD47EixExQMyzMNX6UKiYawNEq3tvNbnEouS13sA9JRT
			hex!["e06451276a99142a02e432405f82f5ec284bde9360921c43cc73bab7f5ee951a"].into(),
			// 5HjY9yUaxj7u4aYK6GYCmVP6mDFFCPU8Z16GpPU9bjcPfdTy
			hex!["facb1457391cf7d029ae7482ea9ecfc49bd2cb4176efc25e9151ce49cb478d5c"]
				.unchecked_into(),
			// 5HFgFtZMUk7Yv4BcSpbWF8j1RqrumxN8fZoWzf1UWiaeXAB2
			hex!["e58b7b5a991f561f1213638553954f9aca8bbe625f35971fdffa91b690063c78"]
				.unchecked_into(),
			// 5CynVah4VHoeNSFupVZSKDPDbvFRy9uUvnXLdFQ2FyaDUTuw
			hex!["287b9d321a7c64ef7a1ae467b443356c78b1fd37b325a53cd4f7920a8294617f"]
				.unchecked_into(),
		),
		(
			// 5FL653wqgXtmno7PJigHBPcAUtr7M8DQyBspTdb15jNKQ8TT
			hex!["907028a2072ff5b922b30305baa9a359a87192bdab488298bced14079c38992d"].into(),
			// 5EA6SpyaHMMCZm74i78qT6X8RHZ2xuQjS6QrdzGGd2kiZH3M
			hex!["5c94bc30c6300c66970e8a05e4a95195de517212d3cccfc1e2b0966c2fd26268"].into(),
			// 5FxPYqwKy4rGgqZyXB8pqZKjtfZa3UpHpk6uemkaPNQGEtpE
			hex!["ac1fd55e8d2706306275909704570081b9612affb6638feffcf5e8a2ef28fc48"]
				.unchecked_into(),
			// 5Dy561YHAs85JiaY6L4oV1tcXAHrdjP5mtnyZuZM9rG1orxX
			hex!["542c760b459f49d0831f189ae14ba08c6584c2c92b374553ec3367bc9abb0edd"]
				.unchecked_into(),
			// 5EcVD3iYy1Wkxb3FUmXwwxjNeyZj5Chs5Bz9pTwZXBYVG59C
			hex!["70b5bda06861e8106991dd409b508236df1d819d2e94c96384297d813b27226d"]
				.unchecked_into(),
		),
		(
			// 5CiCaQrGwtMXzhfQi2rbQWcf6XPfxjhbbLcF3weuVSAGirZp
			hex!["1c98c2563a094a89f1c7577012d17af8f80dd61cc1a0e60cdc0669e696ddd23a"].into(),
			// 5FCqupSYNZs3XxSBu8fjdne7ezPB6F2zE9drzSmr34ZxiM46
			hex!["8ae9c6dda4341c6b587eb2a146bc51214a9f4bb5864948e94e01bbefe6271265"].into(),
			// 5EF9MTK62fpUWmVx2XjCsxyyrxLhHHR3bEWWeP6esEpepydw
			hex!["606ebfc9405f27e68d0e241387679e394bf9da444ade9ee7c17830d079f1b06c"]
				.unchecked_into(),
			// 5Cmb27cMCRUQSRDbT8uqjTWLu6qrdZrQrapbbDZiheakCYEe
			hex!["1f2e0bb471dffd5380732da14cf0519f8eb395882ce411e691dd9f287160afdf"]
				.unchecked_into(),
			// 5D4gSt5WyQk59bGmf6cc3EVWv4y2yYjFwt7wLnbzrHCuY7GQ
			hex!["2c377c8e5beea447ec1f0e8b696080c50f2c03a139a1f5e80cfe12a318759722"]
				.unchecked_into(),
		),
		(
			// 5GNiKihwyxX5hBbuD3L9FM6oXrkChxXVxqrNWPJu9J16FoeU
			hex!["beacebcf1b54b6188727f35ad2a66f3697107b411351f757aaa4df77c6da314f"].into(),
			// 5C58kGPtETiWtE7ccnrbQes2tMegbViy6q8ZnsbS4huQkUmH
			hex!["0053c930e46a6da02bd85d23d270afbf6eb3072a8eb6e767bb1147635ae94761"].into(),
			// 5EqfNNQDCppYZNi7Nz9PqPrKAE16h2sRgCX3oTsRe5P2mYge
			hex!["7ac22291f7d2d8f68100a8e0f7cb37dc0819c3c86feeb05891228d98c94fbe59"]
				.unchecked_into(),
			// 5EPUmW3CCgpKmwf9zZAwHdLNcYppeyqNGPfxA6rr2yDah2BS
			hex!["66ca115a1063fd978d9f77acaeec737fecba9bf7bac115595430d41739e663c5"]
				.unchecked_into(),
			// 5DDAHYCmPSU8D3YpKCdRdS4uNJbvPuWqbAL4yTxxrnYpyCNH
			hex!["32af29f7e30cc1a93cad71a53d05451d38b47e586f3479997c0f787a8f501671"]
				.unchecked_into(),
		),
	];

	// generated with secret: ./target/release/node-template key inspect "$secret"/fir
	let root_key: AccountId = hex![
		// 5Ct1SLsTZB4bMeKCdchDWL2oN7hNTpjmrVqDsrWqspERFSDo
		"2413b72fb065b202a7b84c9a663c167c1435ec0814edf40a3c7ccbcf7a23ba33"
	]
	.into();

	let endowed_accounts: Vec<AccountId> = vec![root_key.clone()];

	testnet_genesis(
		wasm_binary,
		initial_authorities,
		vec![],
		root_key,
		endowed_accounts,
		true,
	)
}

/// Configure initial storage state for FRAME modules.
fn testnet_genesis(
	wasm_binary: &[u8],
	initial_authorities: Vec<(AccountId, AccountId, BabeId, GrandpaId, ImOnlineId)>,
	initial_nominators: Vec<AccountId>,
	root_key: AccountId,
	mut endowed_accounts: Vec<AccountId>,
	_enable_println: bool,
) -> GenesisConfig {
	// endow all authorities and nominators.
	initial_authorities
	.iter()
	.map(|x| &x.0)
	.chain(initial_nominators.iter())
	.for_each(|x| {
		if !endowed_accounts.contains(x) {
			endowed_accounts.push(x.clone())
		}
	});

	// stakers: all validators and nominators.
	const ENDOWMENT: Balance = 10_000_000 * DOLLARS;
	const STASH: Balance = ENDOWMENT / 1000;
	let mut rng = rand::thread_rng();
	let stakers = initial_authorities
		.iter()
		.map(|x| (x.0.clone(), x.1.clone(), STASH, StakerStatus::Validator))
		.chain(initial_nominators.iter().map(|x| {
			use rand::{seq::SliceRandom, Rng};
			let limit = (MaxNominations::get() as usize).min(initial_authorities.len());
			let count = rng.gen::<usize>() % limit;
			let nominations = initial_authorities
				.as_slice()
				.choose_multiple(&mut rng, count)
				.into_iter()
				.map(|choice| choice.0.clone())
				.collect::<Vec<_>>();
			(x.clone(), x.clone(), STASH, StakerStatus::Nominator(nominations))
		}))
		.collect::<Vec<_>>();

	GenesisConfig {
		system: SystemConfig {
			// Add Wasm runtime to storage.
			code: wasm_binary.to_vec(),
		},
		balances: BalancesConfig {
			// Configure endowed accounts with initial balance of 1 << 60.
			balances: endowed_accounts.iter().cloned().map(|k| (k, 1 << 60)).collect(),
		},
		babe: BabeConfig {
			authorities: vec![],
			epoch_config: Some(BABE_GENESIS_EPOCH_CONFIG),
		},
		grandpa: GrandpaConfig {
			authorities: vec![],
		},
		session: SessionConfig {
			keys: initial_authorities
				.iter()
				.map(|x| {
					(
						x.0.clone(),
						x.0.clone(),
						session_keys(x.2.clone(), x.3.clone(), x.4.clone()),
					)
				})
				.collect::<Vec<_>>(),
		},
		staking: StakingConfig {
			validator_count: initial_authorities.len() as u32,
			minimum_validator_count: initial_authorities.len() as u32,
			invulnerables: initial_authorities.iter().map(|x| x.0.clone()).collect(),
			slash_reward_fraction: Perbill::from_percent(10),
			stakers,
			..Default::default()
		},
		im_online: ImOnlineConfig { keys: vec![] },
		sudo: SudoConfig {
			// Assign network admin rights.
			key: Some(root_key),
		},
		transaction_payment: Default::default(),
	}
}
