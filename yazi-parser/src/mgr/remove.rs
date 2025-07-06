use yazi_shared::{event::CmdCow, url::Url};

pub struct RemoveOpt {
	pub force:       bool,
	pub permanently: bool,
	pub hovered:     bool,
	pub targets:     Vec<Url>,
}

impl From<CmdCow> for RemoveOpt {
	fn from(mut c: CmdCow) -> Self {
		Self {
			force:       c.bool("force"),
			permanently: c.bool("permanently"),
			hovered:     c.bool("hovered"),
			targets:     c.take_any("targets").unwrap_or_default(),
		}
	}
}
