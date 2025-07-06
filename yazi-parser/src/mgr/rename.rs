use std::borrow::Cow;

use yazi_shared::event::CmdCow;

pub struct RenameOpt {
	pub hovered: bool,
	pub force:   bool,
	pub empty:   Cow<'static, str>,
	pub cursor:  Cow<'static, str>,
}

impl From<CmdCow> for RenameOpt {
	fn from(mut c: CmdCow) -> Self {
		Self {
			hovered: c.bool("hovered"),
			force:   c.bool("force"),
			empty:   c.take_str("empty").unwrap_or_default(),
			cursor:  c.take_str("cursor").unwrap_or_default(),
		}
	}
}
