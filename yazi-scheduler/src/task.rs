use tokio::sync::mpsc;

#[derive(Debug, Default)]
pub struct Task {
	pub id:    usize,
	pub name:  String,
	pub stage: TaskStage,
	pub kind:  TaskKind,

	pub total: u32,
	pub succ:  u32,
	pub fail:  u32,

	pub found:     u64,
	pub processed: u64,

	pub logs:   String,
	pub logger: Option<mpsc::UnboundedSender<String>>,
}

impl Task {
	pub fn new(id: usize, name: String) -> Self { Self { id, name, ..Default::default() } }
}

#[derive(Debug)]
pub struct TaskSummary {
	pub name: String,

	pub total: u32,
	pub succ:  u32,
	pub fail:  u32,

	pub found:     u64,
	pub processed: u64,
}

impl From<&Task> for TaskSummary {
	fn from(task: &Task) -> Self {
		TaskSummary {
			name: task.name.clone(),

			total: task.total,
			succ:  task.succ,
			fail:  task.fail,

			found:     task.found,
			processed: task.processed,
		}
	}
}

#[derive(Debug)]
pub enum TaskOp {
	// id, size
	New(usize, u64),
	// id, processed, size
	Adv(usize, u32, u64),
	// id
	Succ(usize),
	// id
	Fail(usize, String),
	// id, line
	Log(usize, String),
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Ord, PartialOrd)]
pub enum TaskStage {
	#[default]
	Pending,
	Dispatched,
	Hooked,
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Ord, PartialOrd)]
pub enum TaskKind {
	#[default]
	User,
	PreCache,
}
