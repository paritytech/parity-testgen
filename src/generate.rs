use super::{Account, Action, ActionKind, Params, Secret};
use super::rpc::Client;
use super::scheduler::Scheduler;

use ethkey::{KeyPair, Generator, Random};
use ethstore::{EthStore, SecretStore};
use time::{self, Duration, Tm};
use rand::{Rng, OsRng};

use std::cell::RefCell;
use std::process::{Command, Stdio};
use std::thread;

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

	// run the simulation, blocking until it stops.
	fn run_until(&mut self, end: Tm) -> Vec<Action> {
		let mut last = self.start;
		let mut now = time::now();
		let actions = RefCell::new(Vec::new());

		{
			let mut scheduler = Scheduler::default();

			scheduler.once_every(Duration::milliseconds(10), || {
				if self.rng.gen::<f32>() <= CREATE_ACCOUNT_CHANCE {
					const PASS_LEN: usize = 20;

					let pair = Random.generate().expect("failed to generate keypair");
					let secret: Secret = pair.secret().clone().into();
					let address = pair.address().into();
					let pass = ::random_ascii_lowercase(PASS_LEN);

					self.store.insert_account(secret.0.clone(), &pass).expect("failed to insert account");
					let account = Account::new(address, secret, pass);
					actions.borrow_mut().push(Action::new(ActionKind::CreateAccount(account.clone()), time::now() - self.start));

					// have the first account be a miner.
					if self.users.is_empty() && self.miners.is_empty() {
						self.client.set_author(account.address());
						self.miners.push(account);
					} else if self.rng.gen::<f32>() <= MINER_PROPORTION {
						self.miners.push(account);
					} else {
						self.users.push(account);
					}
				}
			});

			while now < end {
				let dt = now - last;
				last = now;
				now = time::now();

				scheduler.tick(dt);
			}
		}

		actions.into_inner()
	}
}

/// Generate a test using random processes.
///
/// Sends output to stdout.
pub fn generate(params: Params) {
	let run_for = Duration::seconds(params.args.flag_time as i64);
	let start = time::now();
	let end = start + run_for;

	println!("Executing parity");
	// todo: set Stdout, etc.
	let mut parity_child = params.parity_command()
		.stdout(Stdio::null())
		.stderr(Stdio::null())
		.spawn().unwrap();

	let mut ethminer_child = Command::new("ethminer")
		.stdout(Stdio::null())
		.stderr(Stdio::null())
		.spawn().unwrap();

	let mut sim = Simulation::new(start, params.key_store);

	sim.run_until(end);

	println!("Ending simulation");
	let _ = parity_child.kill();
	let _ = ethminer_child.kill();
}