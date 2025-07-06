use yazi_shared::event::CmdCow;

pub struct HardlinkOpt {
	pub force:  bool,
	pub follow: bool,
}

impl From<CmdCow> for HardlinkOpt {
	fn from(c: CmdCow) -> Self { Self { force: c.bool("force"), follow: c.bool("follow") } }
}
