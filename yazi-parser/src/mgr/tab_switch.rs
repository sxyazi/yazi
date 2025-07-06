use yazi_shared::event::{CmdCow, Data};

pub struct TabSwitchOpt {
	pub step:     isize,
	pub relative: bool,
}

impl From<CmdCow> for TabSwitchOpt {
	fn from(c: CmdCow) -> Self {
		Self { step: c.first().and_then(Data::as_isize).unwrap_or(0), relative: c.bool("relative") }
	}
}
