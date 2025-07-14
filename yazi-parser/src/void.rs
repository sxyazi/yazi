use yazi_shared::event::CmdCow;

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
pub struct VoidOpt;

impl From<CmdCow> for VoidOpt {
	fn from(_: CmdCow) -> Self { Self }
}

impl From<()> for VoidOpt {
	fn from(_: ()) -> Self { Self }
}
