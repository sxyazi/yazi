use yazi_shared::event::CmdCow;

pub struct VisualModeOpt {
	pub unset: bool,
}

impl From<CmdCow> for VisualModeOpt {
	fn from(c: CmdCow) -> Self { Self { unset: c.bool("unset") } }
}
