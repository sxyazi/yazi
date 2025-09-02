use crate::Task;

#[derive(Debug, PartialEq, Eq)]
pub struct TaskSnap {
	pub name: String,
}

impl From<&Task> for TaskSnap {
	fn from(task: &Task) -> Self { TaskSnap { name: task.name.clone() } }
}
