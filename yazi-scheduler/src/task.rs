use std::ops::Deref;

use tokio::sync::mpsc;
use yazi_shared::id::Id;

use crate::{TaskHandle, TaskIn, TaskProg, hook::HookIn};

#[derive(Debug)]
pub struct Task {
	pub(crate) handle: TaskHandle,
	pub title:         String,
	pub(crate) prog:   TaskProg,
	pub(crate) hook:   Option<HookIn>,

	pub logs:   String,
	pub logger: Option<mpsc::UnboundedSender<String>>,
}

impl Deref for Task {
	type Target = TaskHandle;

	fn deref(&self) -> &Self::Target { &self.handle }
}

impl Task {
	pub(super) fn new(id: Id, title: String, prog: TaskProg) -> Self {
		Self {
			handle: TaskHandle::new(id),
			title,
			prog,
			hook: None,

			logs: Default::default(),
			logger: Default::default(),
		}
	}

	pub(crate) fn log(&mut self, line: String) {
		self.logs.push_str(&line);
		self.logs.push('\n');

		if let Some(logger) = &self.logger {
			logger.send(line).ok();
		}
	}

	pub(super) fn with_hook(&mut self, hook: impl Into<HookIn>) -> &mut Self {
		let mut hook = hook.into();
		hook.set_id(self.id);

		self.hook = Some(hook);
		self
	}
}
