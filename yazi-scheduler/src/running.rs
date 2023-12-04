use std::collections::BTreeMap;

use futures::future::BoxFuture;
use yazi_config::TASKS;

use super::{Task, TaskStage};
use crate::TaskKind;

#[derive(Default)]
pub struct Running {
	incr: usize,

	pub(super) hooks:
		BTreeMap<usize, Box<dyn (FnOnce(bool) -> BoxFuture<'static, ()>) + Send + Sync>>,
	pub(super) all:   BTreeMap<usize, Task>,
}

impl Running {
	pub(super) fn add(&mut self, name: String) -> usize {
		self.incr += 1;
		self.all.insert(self.incr, Task::new(self.incr, name));
		self.incr
	}

	#[inline]
	pub fn get(&self, id: usize) -> Option<&Task> { self.all.get(&id) }

	#[inline]
	pub fn get_mut(&mut self, id: usize) -> Option<&mut Task> { self.all.get_mut(&id) }

	#[inline]
	pub fn get_id(&self, idx: usize) -> Option<usize> { self.values().nth(idx).map(|t| t.id) }

	#[inline]
	pub fn len(&self) -> usize { self.all.len() }

	#[inline]
	pub(super) fn exists(&self, id: usize) -> bool { self.all.contains_key(&id) }

	#[inline]
	pub fn values(&self) -> impl Iterator<Item = &Task> {
		let map = self.all.values();
		if TASKS.ignore_precaching_tasks {
			map.into_iter().filter(|t| t.kind == TaskKind::User).collect::<Vec<_>>().into_iter()
		} else {
			map.into_iter().collect::<Vec<_>>().into_iter()
		}
	}

	#[inline]
	pub fn is_empty(&self) -> bool { self.all.is_empty() }

	pub(super) fn try_remove(
		&mut self,
		id: usize,
		stage: TaskStage,
	) -> Option<BoxFuture<'static, ()>> {
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
