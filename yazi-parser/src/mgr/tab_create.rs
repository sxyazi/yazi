use yazi_boot::BOOT;
use yazi_fs::expand_path;
use yazi_shared::{event::CmdCow, url::Url};

pub struct TabCreateOpt {
	pub wd: Option<Url>,
}

impl From<CmdCow> for TabCreateOpt {
	fn from(mut c: CmdCow) -> Self {
		if c.bool("current") {
			return Self { wd: None };
		}
		let Some(mut wd) = c.take_first_url() else {
			return Self { wd: Some(Url::from(&BOOT.cwds[0])) };
		};
		if wd.is_regular() && !c.bool("raw") {
			wd = Url::from(expand_path(wd));
		}
		Self { wd: Some(wd) }
	}
}
