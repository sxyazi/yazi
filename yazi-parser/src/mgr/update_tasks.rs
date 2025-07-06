use yazi_shared::{event::CmdCow, url::Url};

pub struct UpdateTasksOpt {
	pub urls: Vec<Url>,
}

impl TryFrom<CmdCow> for UpdateTasksOpt {
	type Error = ();

	fn try_from(mut c: CmdCow) -> Result<Self, Self::Error> {
		Ok(Self { urls: c.take_any("urls").ok_or(())? })
	}
}
