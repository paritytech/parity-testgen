extern crate docopt;
extern crate ethkey;
extern crate ethstore;
extern crate hyper;
extern crate rand;
extern crate rustc_serialize;
extern crate serde;
extern crate serde_json;
extern crate time;

use std::path::PathBuf;
use std::process::Command;

use ethkey::{Address, KeyPair, Secret};
use ethstore::{DiskDirectory, EthStore};

mod generate;
mod replay;
mod rpc;

const USAGE: &'static str = "
Parity-testgen
Automatic parity test generation and replay.

Usage:
  parity-testgen --replay <file>
  parity-testgen --parity <string>
  parity-testgen --time <num> [options]
  parity-testgen (-h | --help)
  parity-testgen [options]

Options:
  -h --help         Show this screen.
  --replay FILE     Replay a test from a given file.
  --parity FILE     The parity executable to run.
  --time SECONDS    The amount of time to spend generating a test.
                    Note that the blocktime is 15 seconds. [default: 900]
";

const DEFAULT_CHAIN: &'static str = include_str!("chainspec.json");

#[derive(Debug, RustcDecodable)]
struct Args {
	flag_replay: Option<String>,
	flag_parity: Option<String>,
	flag_time: usize,
}

/// Account metadata. This is created using a KeyStore shared by parity_testgen
/// and parity itself.
struct Account {
	address: Address,
	secret: Secret,
	pass: String,
}

impl Account {
	/// Create a new account.
	pub fn new(addr: Address, secret: Secret, pass: String) -> Account {
		Account {
			address: addr,
			secret: secret,
			pass: pass,
		}
	}

	/// Get the accont's address.
	pub fn address(&self) -> Address { self.address.clone() }

	/// Get the account's secret key.
	pub fn secret(&self) -> Secret { self.secret.clone() }

	/// Get the account's password.
	pub fn pass(&self) -> String { self.pass.clone() }
}

fn random_ascii_lowercase(len: usize) -> String {
	use rand::Rng;

	let mut rng = rand::thread_rng();
	(0..len).map(|_| (rng.gen::<u8>() % 26 + 97) as char).collect()
}

// actions which can be taken in the log file.
enum Action {
	// account details,
	CreateAccount(Account),
	// "retire" an account, making it go dormant
	RetireAccount(Address),
	// a block was mined, this will be some raw data for the replay.
	BlockMined(Vec<u8>),
}

/// Keeps track of the directories we need to keep track of.
struct Directories {
	root: PathBuf,
}

impl Directories {
	/// With a new, temporary, random root.
	pub fn temp_random() -> Self {
		const PATH_LEN: usize = 12;

		let mut temp_dir = ::std::env::temp_dir();
		let random_dir: String = random_ascii_lowercase(PATH_LEN);
		temp_dir.push(&random_dir);
		Directories::with_root(temp_dir)
	}

	/// With a given root.
	pub fn with_root(root: PathBuf) -> Self {
		Directories { root: root }
	}

	/// Get the root directory.
	pub fn root(&self) -> PathBuf {
		self.root.clone()
	}

	/// Get the keys tore directory.
	pub fn keys(&self) -> PathBuf {
		let mut keys = self.root.clone();
		keys.push("keys");
		keys
	}

	/// Get the datbase directory.
	pub fn db(&self) -> PathBuf {
		let mut db = self.root.clone();
		db.push("db");
		db
	}
}

/// Parameters to generation and replay functions.
pub struct Params {
	dirs: Directories,
	key_store: EthStore,
	args: Args,
	// configuration soon
}

impl Params {
	// initialize the parameters from a directories structure.
	fn from_directories(dirs: Directories, args: Args) -> Self {
		let disk_directory = match DiskDirectory::create(dirs.keys()) {
			Ok(dd) => dd,
			Err(e) => panic!("Failed to create key store: {}", e),
		};

		Params {
			dirs: dirs,
			key_store: EthStore::open(Box::new(disk_directory)).expect("Failed to open key store directory."),
			args: args,
		}
	}

	// get the command to run parity.
	fn parity_command(&self) -> Command {
		let mut c = Command::new(self.args.flag_parity.clone().unwrap_or("parity".into()));
		c.arg("--keys-path").arg(self.dirs.keys());
		c.arg("--db-path").arg(self.dirs.db());

		c
	}
}

fn main() {
	let args: Args = docopt::Docopt::new(USAGE).and_then(|d| d.decode()).unwrap_or_else(|e| e.exit());

	let params = Params::from_directories(Directories::temp_random(), args);
	if let Some(file) = params.args.flag_replay.clone() {
		::replay::replay(file.into(), params);
	} else {
		::generate::generate(params);
	}
}