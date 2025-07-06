use yazi_shared::event::CmdCow;

pub struct ShowOpt;

impl From<CmdCow> for ShowOpt {
	fn from(_: CmdCow) -> Self { Self }
}

impl From<()> for ShowOpt {
	fn from(_: ()) -> Self { Self }
}
