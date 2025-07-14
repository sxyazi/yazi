use anyhow::bail;
use serde::Serialize;
use yazi_shared::event::CmdCow;

pub struct UpdateProgressOpt {
	pub progress: TasksProgress,
}

impl TryFrom<CmdCow> for UpdateProgressOpt {
	type Error = anyhow::Error;

	fn try_from(mut c: CmdCow) -> Result<Self, Self::Error> {
		let Some(progress) = c.take_any("progress") else {
			bail!("Invalid 'progress' in UpdateProgressOpt");
		};

		Ok(Self { progress })
	}
}

// --- Progress
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Serialize)]
pub struct TasksProgress {
	pub total: u32,
	pub succ:  u32,
	pub fail:  u32,

	pub found:     u64,
	pub processed: u64,
}
