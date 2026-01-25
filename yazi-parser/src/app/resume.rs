use anyhow::bail;
use yazi_shared::{CompletionToken, event::CmdCow};

pub struct ResumeOpt {
	pub token: CompletionToken,
}

impl TryFrom<CmdCow> for ResumeOpt {
	type Error = anyhow::Error;

	fn try_from(mut c: CmdCow) -> Result<Self, Self::Error> {
		let Some(token) = c.take_any("token") else {
			bail!("Invalid 'token' in ResumeOpt");
		};

		Ok(Self { token })
	}
}
