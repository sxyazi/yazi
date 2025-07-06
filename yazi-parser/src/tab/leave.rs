use yazi_shared::event::CmdCow;

pub struct LeaveOpt;

impl From<()> for LeaveOpt {
	fn from(_: ()) -> Self { Self }
}

impl From<CmdCow> for LeaveOpt {
	fn from(_: CmdCow) -> Self { Self }
}
