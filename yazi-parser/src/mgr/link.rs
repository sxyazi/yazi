use yazi_shared::event::CmdCow;

pub struct LinkOpt {
	pub relative: bool,
	pub force:    bool,
}

impl From<CmdCow> for LinkOpt {
	fn from(c: CmdCow) -> Self { Self { relative: c.bool("relative"), force: c.bool("force") } }
}
