use super::{Account, Action, ActionKind, Params, Secret};
use super::rpc::Client;
use super::scheduler::Scheduler;

use ethkey::{Generator, Random};
use ethstore::{EthStore, SecretStore};
use time::{self, Duration, Tm};
use rand::{Rng, OsRng};

use std::cell::{RefCell, RefMut};
use std::process::{Child, Command, Stdio};

// amount of time to wait for parity to start up.
const STARTUP_TIME_SECONDS: u64 = 3;

/// Manages simulation data.
struct Simulation {
	actions: RefCell<Vec<Action>>,
	store: RefCell<EthStore>,
	users: RefCell<Vec<Account>>,
	miners: RefCell<Vec<Account>>,
	client: RefCell<Client>,
	start: Tm,
	rng: RefCell<OsRng>,
}

impl Simulation {
	fn new(start: Tm, store: EthStore) -> Self {
		Simulation {
			actions: RefCell::new(Vec::new()),
			store: RefCell::new(store),
			users: RefCell::new(Vec::new()),
			miners: RefCell::new(Vec::new()),
			client: RefCell::new(Client::new()),
			start: start,
			rng: RefCell::new(OsRng::new().expect("failed to initialize rng")),
		}
	}

	// helpers for refcell borrowing.

	fn actions(&self) -> RefMut<Vec<Action>> { self.actions.borrow_mut() }
	fn store(&self) -> RefMut<EthStore> { self.store.borrow_mut() }
	fn users(&self) -> RefMut<Vec<Account>> { self.users.borrow_mut() }
	fn miners(&self) -> RefMut<Vec<Account>> { self.miners.borrow_mut() }
	fn client(&self) -> RefMut<Client> { self.client.borrow_mut() }
	fn rng(&self) -> RefMut<OsRng> { self.rng.borrow_mut() }

	// run the simulation, blocking until it stops.
	fn run_until(self, end: Tm) -> Vec<Action> {
		let mut last = self.start;
		let mut now = time::now();

		{
			let mut scheduler = Scheduler::default();

			scheduler.once_every(Duration::milliseconds(10), || self.account_creation());

			// change the author once every 5 seconds.
			scheduler.once_every(Duration::seconds(5), || self.change_author());

			while now < end {
				let dt = now - last;
				last = now;
				now = time::now();

				scheduler.tick(dt);
			}
		}

		self.actions.into_inner()
	}

	// account creation routine
	fn account_creation(&self) {
		// chance to create an account on a given tick.
		const CREATE_ACCOUNT_CHANCE: f32 = 0.025;

		// chance that a created account is a miner
		const MINER_PROPORTION: f32 = 0.4;

		let mut actions = self.actions();
		let mut store = self.store();
		let mut rng = self.rng();
		let mut users = self.users();
		let mut miners = self.miners();
		let mut client = self.client();

		if rng.gen::<f32>() <= CREATE_ACCOUNT_CHANCE {
			const PASS_LEN: usize = 20;

			let pair = Random.generate().expect("failed to generate keypair");
			let secret: Secret = pair.secret().clone().into();
			let address = pair.address().into();
			let pass = ::random_ascii_lowercase(PASS_LEN);

			store.insert_account(secret.0.clone(), &pass).expect("failed to insert account");
			let account = Account::new(address, secret, pass);
			actions.push(Action::new(ActionKind::CreateAccount(account.clone()), time::now() - self.start));

			// have the first account be a miner.
			if users.is_empty() && miners.is_empty() {
				client.set_author(account.address()).unwrap();
				miners.push(account.clone());

				actions.push(Action::new(ActionKind::SetAuthor(account.address()), time::now() - self.start));
			} else if rng.gen::<f32>() <= MINER_PROPORTION {
				miners.push(account);
			} else {
				users.push(account);
			}
		}
	}

	// block author change routine.
	fn change_author(&self) {
		let mut rng = self.rng();
		let miners = self.miners();

		let idx = rng.gen::<usize>() % miners.len();
		let acc = miners[idx].clone();

		self.client().set_author(acc.address()).unwrap();
		self.actions().push(Action::new(ActionKind::SetAuthor(acc.address()), time::now() - self.start));
	}
}

// panic guard for killing the child processes.
struct ChildKiller {
	parity: Child,
	ethminer: Child,
}

impl Drop for ChildKiller {
	fn drop(&mut self) {
		let _ = self.parity.kill();
		let _ = self.ethminer.kill();
	}
}

/// Generate a test using random processes.
///
/// Produces a vector of actions which occurred.
pub fn generate(params: Params) -> Vec<Action> {
	let run_for = Duration::seconds(params.args.flag_time as i64);

	println!("Executing parity");
	// todo: set Stdout, etc.
	let parity_child = params.parity_command()
		.stdout(Stdio::null())
		.stderr(Stdio::null())
		.spawn().unwrap();

	::std::thread::sleep(::std::time::Duration::from_secs(STARTUP_TIME_SECONDS));

	println!("Executing ethminer");
	let ethminer_child = Command::new("ethminer")
		.stdout(Stdio::null())
		.stderr(Stdio::null())
		.spawn().unwrap();

	let child_killer = ChildKiller {
		parity: parity_child,
		ethminer: ethminer_child,
	};

	let start = time::now();
	let end = start + run_for;

	let mut sim = Simulation::new(start, params.key_store);

	let actions = sim.run_until(end);

	println!("Ending simulation");
	drop(child_killer);

	actions
}