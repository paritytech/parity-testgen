//! Event scheduler for the generator.
use std::collections::VecDeque;

/// A handle for removing tasks.
pub struct Handle(usize);

// the mode of the task
enum Mode {
	// repeated every time this amount of milliseconds has passed.
	Repeated(i64),
	// execute once, with an initial delay of the given amount of millis.
	Delay,
}

struct SchedulerTask<'a> {
	mode: Mode,
	inner: Box<FnMut() + 'a>,
	time_left: i64,
}

#[derive(Default)]
pub struct Scheduler<'a> {
	unused_ids: VecDeque<usize>,
	tasks: Vec<Option<SchedulerTask<'a>>>,
}

impl<'a> Scheduler<'a> {
	/// tick the scheduler
	pub fn tick(&mut self, millis: i64) {
		assert!(millis >= 0, "attempted to tick backwards in time!");
		let mut remove = Vec::new();

		for (idx, task) in self.tasks.iter_mut().enumerate() {
			let task = match task.as_mut() {
				Some(t) => t,
				None => continue,
			};

			task.time_left -= millis;
			if task.time_left <= 0 {
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

	/// Run a task approximately every `millis` milliseconds.
	/// It will be run for the first time `millis` milliseconds after it is submitted.
	pub fn once_every<F: FnMut() + 'a>(&mut self, millis: i64, f: F) -> Handle {
		assert!(millis > 0, "created task with zero or less time between");

		self.add_task(SchedulerTask {
			mode: Mode::Repeated(millis),
			inner: Box::new(f),
			time_left: millis,
		})
	}

	/// Run a task once, delayed by approximately `millis` milliseconds.
	pub fn delay_by<F: FnMut() + 'a>(&mut self, millis: i64, f: F) -> Handle {
		self.add_task(SchedulerTask {
			mode: Mode::Delay,
			inner: Box::new(f),
			time_left: millis,
		})
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