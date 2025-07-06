use yazi_shared::event::CmdCow;

pub struct FollowOpt;

impl From<CmdCow> for FollowOpt {
	fn from(_: CmdCow) -> Self { Self }
}
