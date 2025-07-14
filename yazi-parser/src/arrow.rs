use anyhow::bail;
use yazi_shared::event::CmdCow;

use crate::Step;

#[derive(Default)]
pub struct ArrowOpt {
	pub step: Step,
}

impl TryFrom<CmdCow> for ArrowOpt {
	type Error = anyhow::Error;

	fn try_from(c: CmdCow) -> Result<Self, Self::Error> {
		let Some(step) = c.first() else {
			bail!("'step' is required for ArrowOpt");
		};

		Ok(Self { step: step.try_into()? })
	}
}

impl From<isize> for ArrowOpt {
	fn from(n: isize) -> Self { Self { step: n.into() } }
}
