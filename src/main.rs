extern crate docopt;
extern crate ethkey;
extern crate ethstore;
extern crate rand;
extern crate rustc_serialize;

use std::path::PathBuf;

use ethkey::{Address, KeyPair, Secret};
use ethstore::{DiskDirectory, EthStore};

mod generate;
mod replay;

const USAGE: &'static str = "
Parity-testgen
Automatic parity test generation and replay.

Usage:
  parity-testgen --replay <file>
  parity-testgen (-h | --help)
  parity-testgen [options]

Options:
  -h --help         Show this screen.
  --replay FILE     Replay a test from a given file.
";

const DEFAULT_CHAIN: &'static str = include_str!("chainspec.json");

#[derive(Debug, RustcDecodable)]
struct Args {
	flag_replay: Option<String>,
}

// actions which can be taken in the log file.
enum Actions {
	// account details,
	CreateAccount(Address, Secret, String),
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
		use rand::Rng;

		const PATH_LEN: usize = 12;

		let mut rng = rand::thread_rng();
		let mut temp_dir = ::std::env::temp_dir();
		let random_dir: String = (0..PATH_LEN).map(|_| (rng.gen::<u8>() % 26 + 97) as char).collect();
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
struct Params {
	dirs: Directories,
	key_store: EthStore,
	// configuration soon
}

impl Params {
	// initialize the parameters from a directories structure.
	fn from_directories(dirs: Directories) -> Self {
		let disk_directory = DiskDirectory::create(dirs.keys());
		Params {
			dirs: dirs,
			key_store: EthStore::open(Box::new(disk_directory)).expect("Failed to open key store directory.");
		}
	}
}

fn main() {
	let args: Args = docopt::Docopt::new(USAGE).and_then(|d| d.decode()).unwrap_or_else(|e| e.exit());

	let params = Params::from_directories(Directories::temp_random());
	if let Some(file) = args.flag_replay {
		::replay::replay(file.into(), params);
	} else {
		::generate::generate(params);
	}
}