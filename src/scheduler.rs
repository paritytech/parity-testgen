//! Event scheduler for the generator.
use std::collections::VecDeque;

use time::Duration;

/// A handle for removing tasks.
pub struct Handle(usize);

// the mode of the task
enum Mode {
	// repeated every time this amount of milliseconds has passed.
	Repeated(Duration),
	// execute once, with an initial delay of the given amount of millis.
	Delay,
}

struct SchedulerTask<'a> {
	mode: Mode,
	inner: Box<FnMut() + 'a>,
	time_left: Duration,
}

#[derive(Default)]
pub struct Scheduler<'a> {
	unused_ids: VecDeque<usize>,
	tasks: Vec<Option<SchedulerTask<'a>>>,
}

impl<'a> Scheduler<'a> {
	/// tick the scheduler
	pub fn tick(&mut self, dt: Duration) {
		assert!(dt > Duration::zero());

		let mut remove = Vec::new();

		for (idx, task) in self.tasks.iter_mut().enumerate() {
			let task = match task.as_mut() {
				Some(t) => t,
				None => continue,
			};

			let new_time = task.time_left - dt;
			if new_time > Duration::zero() {
				task.time_left = new_time;
			} else {
				(task.inner)();
				match task.mode {
					Mode::Repeated(how_often) => {
						task.time_left = how_often;
					}
					Mode::Delay => {
						remove.push(idx);
					}
				}
			}
		}

		for idx in remove {
			self.tasks[idx] = None;
			self.unused_ids.push_back(idx);
		}
	}

	/// Run a task approximately every `dt` duration..
	/// It will be run for the first time `dt` after it is first submitted.
	pub fn once_every<F: FnMut() + 'a>(&mut self, dt: Duration, f: F) -> Handle {
		self.add_task(SchedulerTask {
			mode: Mode::Repeated(dt),
			inner: Box::new(f),
			time_left: dt,
		})
	}

	/// Run a task once, delayed by approximately the given duration.
	pub fn delay_by<F: FnMut() + 'a>(&mut self, dt: Duration, f: F) -> Handle {
		self.add_task(SchedulerTask {
			mode: Mode::Delay,
			inner: Box::new(f),
			time_left: dt,
		})
	}

	/// Remove a task by handle.
	pub fn remove(&mut self, handle: Handle) {
		self.tasks[handle.0] = None;
	}

	// add a task to the scheduler
	fn add_task(&mut self, task: SchedulerTask<'a>) -> Handle {
		if let Some(idx) = self.unused_ids.pop_front() {
			self.tasks[idx] = Some(task);
			Handle(idx)
		} else {
			self.tasks.push(Some(task));
			Handle(self.tasks.len() - 1)
		}
	}
}