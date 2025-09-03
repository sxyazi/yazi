use anyhow::bail;
use ordered_float::OrderedFloat;
use serde::Serialize;
use yazi_shared::event::CmdCow;

pub struct UpdateProgressOpt {
	pub summary: TaskSummary,
}

impl TryFrom<CmdCow> for UpdateProgressOpt {
	type Error = anyhow::Error;

	fn try_from(mut c: CmdCow) -> Result<Self, Self::Error> {
		let Some(summary) = c.take_any("summary") else {
			bail!("Invalid 'summary' in UpdateProgressOpt");
		};

		Ok(Self { summary })
	}
}

// --- Progress
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Serialize)]
pub struct TaskSummary {
	pub total:   u32,
	pub success: u32,
	pub failed:  u32,
	pub percent: Option<OrderedFloat<f32>>,
}
