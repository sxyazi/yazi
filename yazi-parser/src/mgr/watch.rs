use yazi_shared::event::CmdCow;

pub struct WatchOpt;

impl From<CmdCow> for WatchOpt {
	fn from(_: CmdCow) -> Self { Self }
}

impl From<()> for WatchOpt {
	fn from((): ()) -> Self { Self }
}
