use hashbrown::{HashMap, hash_map::Entry};
use yazi_config::YAZI;
use yazi_shared::{CompletionToken, Id, Ids};

use super::Task;
use crate::{TaskIn, TaskProg, hook::HookIn};

#[derive(Default)]
pub struct Ongoing {
	inner: HashMap<Id, Task>,
}

impl Ongoing {
	pub(super) fn add<T>(&mut self, r#in: &mut T) -> &mut Task
	where
		T: TaskIn,
		T::Prog: Into<TaskProg> + Default,
	{
		static IDS: Ids = Ids::new();
		let id = IDS.next();

		let title = r#in.set_id(id).title().into_owned();
		let prog = T::Prog::default().into();

		self.inner.entry(id).insert(Task::new(id, title, prog)).into_mut()
	}

	pub(super) fn cancel(&mut self, id: Id) -> Option<HookIn> {
		match self.inner.entry(id) {
			Entry::Occupied(mut oe) => {
				let task = oe.get_mut();
				task.done.complete(false);

				if let Some(hook) = task.hook.take() {
					return Some(hook);
				}

				oe.remove();
			}
			Entry::Vacant(_) => {}
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
}
