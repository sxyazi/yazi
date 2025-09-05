use hashbrown::HashMap;
use ordered_float::OrderedFloat;
use yazi_config::YAZI;
use yazi_parser::app::TaskSummary;
use yazi_shared::{Id, Ids};

use super::Task;
use crate::{Hooks, TaskProg};

#[derive(Default)]
pub struct Ongoing {
	pub(super) hooks: Hooks,
	pub(super) all:   HashMap<Id, Task>,
}

impl Ongoing {
	pub(super) fn add<T>(&mut self, name: String) -> Id
	where
		T: Into<TaskProg> + Default,
	{
		static IDS: Ids = Ids::new();

		let id = IDS.next();
		self.all.insert(id, Task::new::<T>(id, name));
		id
	}

	#[inline]
	pub fn get_mut(&mut self, id: Id) -> Option<&mut Task> { self.all.get_mut(&id) }

	pub fn get_id(&self, idx: usize) -> Option<Id> { self.values().nth(idx).map(|t| t.id) }

	pub fn len(&self) -> usize {
		if YAZI.tasks.suppress_preload {
			self.all.values().filter(|&t| t.prog.is_user()).count()
		} else {
			self.all.len()
		}
	}

	#[inline]
	pub fn exists(&self, id: Id) -> bool { self.all.contains_key(&id) }

	pub fn values(&self) -> Box<dyn Iterator<Item = &Task> + '_> {
		if YAZI.tasks.suppress_preload {
			Box::new(self.all.values().filter(|&t| t.prog.is_user()))
		} else {
			Box::new(self.all.values())
		}
	}

	#[inline]
	pub fn is_empty(&self) -> bool { self.len() == 0 }

	pub fn summary(&self) -> TaskSummary {
		let mut summary = TaskSummary::default();
		let mut percent_sum = 0.0f64;
		let mut percent_count = 0;

		for task in self.values() {
			let s: TaskSummary = task.prog.into();
			if s.total == 0 {
				continue;
			}

			summary.total += 1;
			if let Some(p) = s.percent {
				percent_sum += p.0 as f64;
				percent_count += 1;
			}

			if task.prog.running() {
				continue;
			} else if task.prog.success() {
				summary.success += 1;
			} else {
				summary.failed += 1;
			}
		}

		summary.percent = if percent_count == 0 {
			None
		} else {
			Some(OrderedFloat((percent_sum / percent_count as f64) as f32))
		};
		summary
	}
}
