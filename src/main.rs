extern crate docopt;
extern crate ethkey;
extern crate ethstore;
extern crate hyper;
extern crate rand;
extern crate rustc_serialize;
extern crate serde;
extern crate serde_json;
extern crate time;

use std::io::Write;
use std::fmt;
use std::fs::File;
use std::path::PathBuf;
use std::process::Command;

use ethstore::{DiskDirectory, EthStore};
use rustc_serialize::hex::{FromHex, ToHex};

mod action;
mod generate;
mod replay;
mod rpc;
mod scheduler;

pub use ::action::{Account, Action, ActionKind};

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
  --log-file FILE   The file to write actions out to. [default: out.log]
";

const DEFAULT_CHAIN: &'static str = include_str!("chainspec.json");

#[derive(Debug, RustcDecodable)]
struct Args {
	flag_replay: Option<String>,
	flag_parity: Option<String>,
	flag_time: usize,
	flag_log_file: String,
}

fn random_ascii_lowercase(len: usize) -> String {
	use rand::Rng;

	let mut rng = rand::thread_rng();
	(0..len).map(|_| (rng.gen::<u8>() % 26 + 97) as char).collect()
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

	/// Get the keystore directory.
	pub fn keys(&self) -> PathBuf {
		let mut keys = self.root.clone();
		keys.push("keys");
		keys
	}

	/// Get the database directory.
	pub fn db(&self) -> PathBuf {
		let mut db = self.root.clone();
		db.push("db");
		db
	}

	/// Get the chain.json file
	pub fn chain_file(&self) -> PathBuf {
		let mut chain = self.root.clone();
		chain.push("chain.json");
		chain
	}
}

/// Parameters to generation and replay functions.
pub struct Params {
	dirs: Directories,
	key_store: EthStore,
	args: Args,
}

impl Params {
	// initialize the parameters from a directories structure.
	fn from_directories(dirs: Directories, args: Args) -> Self {
		println!("Using directory: {}", dirs.root().to_str().unwrap());

		let disk_directory = match DiskDirectory::create(dirs.keys()) {
			Ok(dd) => dd,
			Err(e) => panic!("Failed to create key store: {}", e),
		};

		let mut chain_file = File::create(dirs.chain_file()).unwrap();
		let _ = write!(chain_file, "{}", DEFAULT_CHAIN);

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
		c.arg("--jsonrpc-apis").arg("eth,ethcore,personal");
		c.arg("--db-path").arg(self.dirs.db());
		c.arg("--chain").arg(self.dirs.chain_file());
		c
	}
}

macro_rules! hash_wrapper {
	($name: ident) => {
		#[derive(Clone)]
		pub struct $name(ethkey::$name);

		impl serde::Serialize for $name {
			fn serialize<S>(&self, serializer: &mut S) -> Result<(), S::Error>
			where S: serde::Serializer {
				let mut hex = "0x".to_owned();
				hex.push_str(self.0.to_hex().as_ref());
				serializer.serialize_str(hex.as_ref())
			}
		}

		impl serde::Deserialize for $name {
			fn deserialize<D>(deserializer: &mut D) -> Result<$name, D::Error>
			where D: serde::Deserializer {
				struct HashVisitor;

				impl serde::de::Visitor for HashVisitor {
					type Value = $name;

					fn visit_str<E>(&mut self, value: &str) -> Result<Self::Value, E> where E: serde::Error {
						let mut data = ::ethkey::$name::default();

						// 0x + len
						if value.len() != 2 + data.len() * 2 {
							return Err(serde::Error::custom("Invalid length."));
						}

						let bytes = try!(value[2..].from_hex().map_err(|_| serde::Error::custom("Invalid hex value.")));
						data.copy_from_slice(&bytes);

						Ok($name(data))
					}

					fn visit_string<E>(&mut self, value: String) -> Result<Self::Value, E> where E: serde::Error {
						self.visit_str(value.as_ref())
					}
				}
				deserializer.deserialize(HashVisitor)
			}
		}

		impl From<ethkey::$name> for $name {
			fn from(inner: ethkey::$name) -> Self {
				$name(inner)
			}
		}

		impl fmt::Display for $name {
			fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
				self.0.fmt(f)
			}
		}
	}
}

hash_wrapper!(Address);
hash_wrapper!(Secret);

fn main() {
	let args: Args = docopt::Docopt::new(USAGE).and_then(|d| d.decode()).unwrap_or_else(|e| e.exit());

	let params = Params::from_directories(Directories::temp_random(), args);
	if let Some(file) = params.args.flag_replay.clone() {
		::replay::replay(file.into(), params);
	} else {
		let log_filename = params.args.flag_log_file.clone();
		let actions = ::generate::generate(params);

		let mut log_file = File::create(log_filename).unwrap();
		let _ = write!(log_file, "{}", ::serde_json::to_value(&actions));
	}
}