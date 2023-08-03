use tokio::sync::mpsc;

#[derive(Debug)]
pub struct Task {
	pub id:    usize,
	pub name:  String,
	pub stage: TaskStage,

	pub found:     u32,
	pub processed: u32,

	pub todo: u64,
	pub done: u64,

	pub logs:   String,
	pub logger: Option<mpsc::UnboundedSender<String>>,
}

#[derive(Debug)]
pub struct TaskSummary {
	pub name: String,

	pub found:     u32,
	pub processed: u32,

	pub todo: u64,
	pub done: u64,
}

impl Task {
	pub fn new(id: usize, name: String) -> Self {
		Self {
			id,
			name,
			stage: Default::default(),

			found: 0,
			processed: 0,

			todo: 0,
			done: 0,

			logs: Default::default(),
			logger: Default::default(),
		}
	}
}

impl Into<TaskSummary> for &Task {
	fn into(self) -> TaskSummary {
		TaskSummary {
			name: self.name.clone(),

			found:     self.found,
			processed: self.processed,

			todo: self.todo,
			done: self.done,
		}
	}
}

#[derive(Debug)]
pub enum TaskOp {
	// task_id, size
	New(usize, u64),
	// task_id, line
	Log(usize, String),
	// task_id, processed, size
	Adv(usize, u32, u64),
	// task_id
	Done(usize),
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Ord, PartialOrd)]
pub enum TaskStage {
	#[default]
	Pending,
	Dispatched,
	Hooked,
}
