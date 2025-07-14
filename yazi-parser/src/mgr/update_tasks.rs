use anyhow::bail;
use yazi_shared::{event::CmdCow, url::Url};

pub struct UpdateTasksOpt {
	pub urls: Vec<Url>,
}

impl TryFrom<CmdCow> for UpdateTasksOpt {
	type Error = anyhow::Error;

	fn try_from(mut c: CmdCow) -> Result<Self, Self::Error> {
		let Some(urls) = c.take_any("urls") else {
			bail!("Invalid 'urls' argument in UpdateTasksOpt");
		};

		Ok(Self { urls })
	}
}
