use anyhow::bail;
use yazi_shared::{SStr, event::CmdCow};

pub struct DeprecateOpt {
	pub content: SStr,
}

impl TryFrom<CmdCow> for DeprecateOpt {
	type Error = anyhow::Error;

	fn try_from(mut c: CmdCow) -> Result<Self, Self::Error> {
		let Ok(content) = c.take("content") else {
			bail!("Invalid 'content' in DeprecateOpt");
		};

		Ok(Self { content })
	}
}
