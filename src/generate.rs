use super::{Account, Action, Params};
use super::rpc::Client;

use ethkey::{KeyPair, Generator, Random};
use ethstore::{EthStore, SecretStore};
use time::{self, Duration, Tm};
use rand::{Rng, OsRng};

use std::thread;

// only wake up every X milliseconds.
const SLEEP_BETWEEN_TICKS_MS: i64 = 500;

// chance to create an account on a given tick.
const CREATE_ACCOUNT_CHANCE: f32 = 0.025;

// chance that a created account is a miner
const MINER_PROPORTION: f32 = 0.4;

/// Manages simulation data.
struct Simulation {
	store: EthStore,
	users: Vec<Account>,
	miners: Vec<Account>,
	client: Client,
	start: Tm,
	rng: OsRng,
	actions: Vec<(Duration, Action)>,
}

impl Simulation {
	fn new(start: Tm, store: EthStore) -> Self {
		Simulation {
			store: store,
			users: Vec::new(),
			miners: Vec::new(),
			client: Client::new(),
			start: start,
			actions: Vec::new(),
			rng: OsRng::new().expect("failed to initialize rng"),
		}
	}

	// tick the simulation.
	//
	// create new accounts, retire old ones, dispatch transactions, etc.
	fn tick(&mut self) {
		if self.rng.gen::<f32>() <= CREATE_ACCOUNT_CHANCE {
			const PASS_LEN: usize = 20;

			let pair = Random.generate().expect("failed to generate keypair");
			let secret = pair.secret().clone();
			let address = pair.address();
			let pass = ::random_ascii_lowercase(PASS_LEN);

			self.store.insert_account(secret.clone(), &pass).expect("failed to insert account");
			let account = Account::new(address, secret, pass);

			if self.rng.gen::<f32>() <= MINER_PROPORTION {
				self.miners.push(account);
			} else {
				self.users.push(account);
			}
		}


	}
}

/// Generate a test using random processes.
///
/// Sends output to stdout.
pub fn generate(params: Params) {
	let sleep_between = Duration::milliseconds(SLEEP_BETWEEN_TICKS_MS).to_std().unwrap();
	let run_for = Duration::seconds(params.args.flag_time as i64);
	let start = time::now();
	let end = start + run_for;

	let mut sim = Simulation::new(start, params.key_store);

	while time::now() < end {
		sim.tick();

		thread::sleep(sleep_between);
	}
}