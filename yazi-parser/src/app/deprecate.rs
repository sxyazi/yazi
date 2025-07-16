use anyhow::bail;
use yazi_shared::event::CmdCow;

pub struct DeprecateOpt {
	pub content: String,
}

impl TryFrom<CmdCow> for DeprecateOpt {
	type Error = anyhow::Error;

	fn try_from(mut c: CmdCow) -> Result<Self, Self::Error> {
		let Some(content) = c.take_str("content") else {
			bail!("Invalid 'content' in DeprecateOpt");
		};

		Ok(Self { content: content.into_owned() })
	}
}
