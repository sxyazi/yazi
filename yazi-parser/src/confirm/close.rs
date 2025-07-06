use yazi_shared::event::CmdCow;

pub struct CloseOpt {
	pub submit: bool,
}

impl From<CmdCow> for CloseOpt {
	fn from(c: CmdCow) -> Self { Self { submit: c.bool("submit") } }
}

impl From<bool> for CloseOpt {
	fn from(submit: bool) -> Self { Self { submit } }
}
