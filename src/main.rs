extern crate docopt;
extern crate ethkey;
extern crate rand;
extern crate rustc_serialize;

use std::path::PathBuf;
use ethkey::{Address, KeyPair, Secret};

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
	// account details
	CreateAccount(Address, Secret),
	// "retire" an account, making it go dormant
	RetireAccount(Address),
	// a block was mined, this will be some raw data for the replay.
	BlockMined(Vec<u8>),
}

/// get the temp directory to place files in.
fn get_temp_dir() -> PathBuf {
	use rand::Rng;

	const PATH_LEN: usize = 12;

	let mut rng = rand::thread_rng();
	let mut temp_dir = ::std::env::temp_dir();
	let random_dir: String = (0..PATH_LEN).map(|_| (rng.gen::<u8>() % 26 + 97) as char).collect();
	temp_dir.push(&random_dir);
	temp_dir
}

fn main() {
	let args: Args = docopt::Docopt::new(USAGE).and_then(|d| d.decode()).unwrap_or_else(|e| e.exit());

	let _temp_dir = get_temp_dir();
	if let Some(file) = args.flag_replay {
		::replay::replay(file.into());
	} else {
		::generate::generate();
	}
}