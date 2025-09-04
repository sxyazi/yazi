use serde::Serialize;

use crate::{Task, TaskProg};

#[derive(Debug, PartialEq, Eq, Serialize)]
pub struct TaskSnap {
	pub name: String,
	pub prog: TaskProg,
}

impl From<&Task> for TaskSnap {
	fn from(task: &Task) -> Self { Self { name: task.name.clone(), prog: task.prog } }
}
