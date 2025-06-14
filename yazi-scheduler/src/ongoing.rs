use std::collections::HashMap;

use futures::future::BoxFuture;
use yazi_config::YAZI;
use yazi_shared::{Id, Ids};

use super::{Task, TaskStage};
use crate::{Hooks, TaskKind};

#[derive(Default)]
pub struct Ongoing {
	pub(super) hooks: Hooks,
	pub(super) all:   HashMap<Id, Task>,
}

impl Ongoing {
	pub fn add(&mut self, kind: TaskKind, name: String) -> Id {
		static IDS: Ids = Ids::new();

		let id = IDS.next();
		self.all.insert(id, Task::new(id, kind, name));
		id
	}

	#[inline]
	pub fn get(&self, id: Id) -> Option<&Task> { self.all.get(&id) }

	#[inline]
	pub fn get_mut(&mut self, id: Id) -> Option<&mut Task> { self.all.get_mut(&id) }

	#[inline]
	pub fn get_id(&self, idx: usize) -> Option<Id> { self.values().nth(idx).map(|t| t.id) }

	#[inline]
	pub fn len(&self) -> usize {
		if YAZI.tasks.suppress_preload {
			self.all.values().filter(|t| t.kind != TaskKind::Preload).count()
		} else {
			self.all.len()
		}
	}

	#[inline]
	pub fn exists(&self, id: Id) -> bool { self.all.contains_key(&id) }

	#[inline]
	pub fn values(&self) -> Box<dyn Iterator<Item = &Task> + '_> {
		if YAZI.tasks.suppress_preload {
			Box::new(self.all.values().filter(|t| t.kind != TaskKind::Preload))
		} else {
			Box::new(self.all.values())
		}
	}

	#[inline]
	pub fn is_empty(&self) -> bool { self.len() == 0 }

	pub fn try_remove(&mut self, id: Id, stage: TaskStage) -> Option<BoxFuture<'static, ()>> {
		if let Some(task) = self.get_mut(id) {
			if stage > task.stage {
				task.stage = stage;
			}

			match task.stage {
				TaskStage::Pending => return None,
				TaskStage::Dispatched => {
					if task.succ < task.total {
						return None;
					}
					if let Some(fut) = self.hooks.run_or_pop(id, false) {
						return Some(fut);
					}
				}
				TaskStage::Hooked => {}
			}

			self.all.remove(&id);
		}
		None
	}
}
