use super::{Action, Params};

use time::{self, Duration, Tm};

use std::thread;

const SLEEP_BETWEEN_TICKS_MS: i64 = 50;

struct Simulation {
	start: Tm,
	actions: Vec<(Duration, Action)>,
}

impl Simulation {
	fn new(start: Tm) -> Self {
		Simulation {
			start: start,
			actions: Vec::new(),
		}
	}

	fn tick(&mut self) { }
}

/// Generate a test using random processes.
///
/// Sends output to stdout.
pub fn generate(params: Params) {
	let sleep_between = Duration::milliseconds(SLEEP_BETWEEN_TICKS_MS).to_std().unwrap();
	let run_for = Duration::seconds(params.args.flag_time as i64);
	let start = time::now();
	let end = start + run_for;

	let mut sim = Simulation::new(start);

	while time::now() < end {
		sim.tick();

		thread::sleep(sleep_between);
	}
}