use yazi_shared::event::CmdCow;

pub struct PasteOpt {
	pub force:  bool,
	pub follow: bool,
}

impl From<CmdCow> for PasteOpt {
	fn from(c: CmdCow) -> Self { Self { force: c.bool("force"), follow: c.bool("follow") } }
}
