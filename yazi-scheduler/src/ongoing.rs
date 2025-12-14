use hashbrown::{HashMap, hash_map::RawEntryMut};
use ordered_float::OrderedFloat;
use yazi_config::YAZI;
use yazi_parser::app::TaskSummary;
use yazi_shared::{CompletionToken, Id, Ids};

use super::Task;
use crate::{TaskIn, TaskProg};

#[derive(Default)]
pub struct Ongoing {
	inner: HashMap<Id, Task>,
}

impl Ongoing {
	pub(super) fn add<T>(&mut self, name: String) -> &mut Task
	where
		T: Into<TaskProg> + Default,
	{
		static IDS: Ids = Ids::new();

		let id = IDS.next();
		self.inner.entry(id).insert(Task::new::<T>(id, name)).into_mut()
	}

	pub(super) fn cancel(&mut self, id: Id) -> Option<TaskIn> {
		match self.inner.raw_entry_mut().from_key(&id) {
			RawEntryMut::Occupied(mut oe) => {
				let task = oe.get_mut();
				task.done.complete(false);

				if let Some(hook) = task.hook.take() {
					return Some(hook);
				}

				oe.remove();
			}
			RawEntryMut::Vacant(_) => {}
		}
		None
	}

	pub(super) fn fulfill(&mut self, id: Id) -> Option<Task> {
		let task = self.inner.remove(&id)?;
		task.done.complete(true);
		Some(task)
	}

	#[inline]
	pub fn get_mut(&mut self, id: Id) -> Option<&mut Task> { self.inner.get_mut(&id) }

	pub fn get_id(&self, idx: usize) -> Option<Id> { self.values().nth(idx).map(|t| t.id) }

	#[inline]
	pub fn get_token(&self, id: Id) -> Option<CompletionToken> {
		self.inner.get(&id).map(|t| t.done.clone())
	}

	pub fn len(&self) -> usize {
		if YAZI.tasks.suppress_preload {
			self.inner.values().filter(|&t| t.prog.is_user()).count()
		} else {
			self.inner.len()
		}
	}

	#[inline]
	pub fn exists(&self, id: Id) -> bool { self.inner.contains_key(&id) }

	#[inline]
	pub fn intact(&self, id: Id) -> bool {
		self.inner.get(&id).is_some_and(|t| t.done.completed() != Some(false))
	}

	pub fn values(&self) -> Box<dyn Iterator<Item = &Task> + '_> {
		if YAZI.tasks.suppress_preload {
			Box::new(self.inner.values().filter(|&t| t.prog.is_user()))
		} else {
			Box::new(self.inner.values())
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
			if s.total == 0 && !task.prog.failed() {
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
