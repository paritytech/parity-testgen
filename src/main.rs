extern crate docopt;
extern crate ethkey;
extern crate rand;
extern crate rustc_serialize;

use std::path::PathBuf;

use ethkey::{Address, KeyPair, Secret};

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

/// get the temp directory to give to parity.
fn get_temp_dir() -> PathBuf {
	let mut temp_dir = ::std::env::temp_dir();
	temp_dir.push(".parity");
	temp_dir
}

// generate a test.
fn generate() {
	unimplemented!();
}

// replay a test from a file.
fn replay_from(_file_path: PathBuf) {
	unimplemented!();
}

fn main() {
	let args: Args = docopt::Docopt::new(USAGE).and_then(|d| d.decode()).unwrap_or_else(|e| e.exit());

	let _temp_dir = get_temp_dir();
	if let Some(file) = args.flag_replay {
		replay_from(file.into());
	}
}