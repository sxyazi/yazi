use std::collections::HashMap;

use futures::future::BoxFuture;
use yazi_config::YAZI;

use super::{Task, TaskStage};
use crate::TaskKind;

#[derive(Default)]
pub struct Ongoing {
	incr: usize,

	pub(super) hooks: HashMap<usize, Box<dyn (FnOnce(bool) -> BoxFuture<'static, ()>) + Send + Sync>>,
	pub(super) all:   HashMap<usize, Task>,
}

impl Ongoing {
	pub fn add(&mut self, kind: TaskKind, name: String) -> usize {
		self.incr += 1;
		self.all.insert(self.incr, Task::new(self.incr, kind, name));
		self.incr
	}

	#[inline]
	pub fn get(&self, id: usize) -> Option<&Task> { self.all.get(&id) }

	#[inline]
	pub fn get_mut(&mut self, id: usize) -> Option<&mut Task> { self.all.get_mut(&id) }

	#[inline]
	pub fn get_id(&self, idx: usize) -> Option<usize> { self.values().nth(idx).map(|t| t.id) }

	#[inline]
	pub fn len(&self) -> usize {
		if YAZI.tasks.suppress_preload {
			self.all.values().filter(|t| t.kind != TaskKind::Preload).count()
		} else {
			self.all.len()
		}
	}

	#[inline]
	pub fn exists(&self, id: usize) -> bool { self.all.contains_key(&id) }

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

	pub fn try_remove(&mut self, id: usize, stage: TaskStage) -> Option<BoxFuture<'static, ()>> {
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
					if let Some(hook) = self.hooks.remove(&id) {
						return Some(hook(false));
					}
				}
				TaskStage::Hooked => {}
			}

			self.all.remove(&id);
		}
		None
	}
}
