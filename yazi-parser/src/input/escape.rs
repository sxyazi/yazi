use yazi_shared::event::CmdCow;

pub struct EscapeOpt;

impl From<CmdCow> for EscapeOpt {
	fn from(_: CmdCow) -> Self { Self }
}

impl From<()> for EscapeOpt {
	fn from(_: ()) -> Self { Self }
}
