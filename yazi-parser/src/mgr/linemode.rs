use anyhow::bail;
use yazi_shared::{SStr, event::CmdCow};

pub struct LinemodeOpt {
	pub new: SStr,
}

impl TryFrom<CmdCow> for LinemodeOpt {
	type Error = anyhow::Error;

	fn try_from(mut c: CmdCow) -> Result<Self, Self::Error> {
		let Some(new) = c.take_first_str() else {
			bail!("a string argument is required for LinemodeOpt");
		};

		if new.is_empty() || new.len() > 20 {
			bail!("Linemode must be between 1 and 20 characters long");
		}

		Ok(Self { new })
	}
}
