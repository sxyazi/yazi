use yazi_shared::event::CmdCow;

#[derive(Default)]
pub struct BackspaceOpt {
	pub under: bool,
}

impl From<CmdCow> for BackspaceOpt {
	fn from(c: CmdCow) -> Self { Self { under: c.bool("under") } }
}

impl From<bool> for BackspaceOpt {
	fn from(under: bool) -> Self { Self { under } }
}
