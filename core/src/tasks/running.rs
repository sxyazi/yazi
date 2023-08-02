use std::collections::BTreeMap;

use futures::future::BoxFuture;

use super::{Task, TaskStage};

#[derive(Default)]
pub(super) struct Running {
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
	pub(super) fn get(&mut self, id: usize) -> Option<&mut Task> { self.all.get_mut(&id) }

	#[inline]
	pub(super) fn len(&self) -> usize { self.all.len() }

	#[inline]
	pub(super) fn exists(&self, id: usize) -> bool { self.all.contains_key(&id) }

	#[inline]
	pub(super) fn values(&self) -> impl Iterator<Item = &Task> { self.all.values() }

	#[inline]
	pub(super) fn is_empty(&self) -> bool { self.all.is_empty() }

	pub(super) fn try_remove(
		&mut self,
		id: usize,
		stage: TaskStage,
	) -> Option<BoxFuture<'static, ()>> {
		if let Some(task) = self.get(id) {
			if stage > task.stage {
				task.stage = stage;
			}

			match task.stage {
				TaskStage::Pending => return None,
				TaskStage::Dispatched => {
					if task.processed < task.found {
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
