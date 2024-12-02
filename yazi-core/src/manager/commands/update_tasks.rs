use yazi_shared::{event::CmdCow, url::Url};

use crate::manager::Manager;

pub struct Opt {
	urls: Vec<Url>,
}

impl TryFrom<CmdCow> for Opt {
	type Error = ();

	fn try_from(mut c: CmdCow) -> Result<Self, Self::Error> {
		Ok(Self { urls: c.take_any("urls").ok_or(())? })
	}
}

impl Manager {
	pub fn update_tasks(&mut self, opt: impl TryInto<Opt>) {
		let Ok(opt) = opt.try_into() else {
			return;
		};

		self.watcher.push_files(opt.urls);
	}
}
