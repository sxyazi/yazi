use yazi_shared::event::CmdCow;

pub struct UnyankOpt;

impl From<CmdCow> for UnyankOpt {
	fn from(_: CmdCow) -> Self { Self }
}

impl From<()> for UnyankOpt {
	fn from(_: ()) -> Self { Self }
}
