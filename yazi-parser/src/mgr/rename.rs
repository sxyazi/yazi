use yazi_shared::{SStr, event::CmdCow};

pub struct RenameOpt {
	pub hovered: bool,
	pub force:   bool,
	pub empty:   SStr,
	pub cursor:  SStr,
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
