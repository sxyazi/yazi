use tokio::sync::mpsc;
use yazi_shared::Id;

use crate::TaskProg;

#[derive(Debug)]
pub struct Task {
	pub id:          Id,
	pub name:        String,
	pub(crate) prog: TaskProg,

	pub logs:   String,
	pub logger: Option<mpsc::UnboundedSender<String>>,
}

impl Task {
	pub(super) fn new<T>(id: Id, name: String) -> Self
	where
		T: Into<TaskProg> + Default,
	{
		Self {
			id,
			name,
			prog: T::default().into(),
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
}
