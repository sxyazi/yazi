use yazi_shared::event::CmdCow;

pub struct CloseOpt;

impl From<CmdCow> for CloseOpt {
	fn from(_: CmdCow) -> Self { Self }
}

impl From<()> for CloseOpt {
	fn from(_: ()) -> Self { Self }
}
