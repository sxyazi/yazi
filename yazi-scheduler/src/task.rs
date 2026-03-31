use tokio::sync::mpsc;
use yazi_shared::{CompletionToken, Id};

use crate::{TaskProg, hook::HookIn};

#[derive(Debug)]
pub struct Task {
	pub id:          Id,
	pub name:        String,
	pub(crate) prog: TaskProg,
	pub(crate) hook: Option<HookIn>,
	pub done:        CompletionToken,

	pub logs:   String,
	pub logger: Option<mpsc::UnboundedSender<String>>,
}

impl Task {
	pub(super) fn new(id: Id, name: String, prog: TaskProg) -> Self {
		Self {
			id,
			name,
			prog,
			hook: None,
			done: Default::default(),

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
		self.hook = Some(hook.into().with_id(self.id));
		self
	}
}
