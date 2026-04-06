use serde::Serialize;

use crate::{Task, TaskProg};

#[derive(Debug, PartialEq, Eq, Serialize)]
pub struct TaskSnap {
	pub title: String,
	pub prog:  TaskProg,
}

impl From<&Task> for TaskSnap {
	fn from(task: &Task) -> Self { Self { title: task.title.clone(), prog: task.prog } }
}
