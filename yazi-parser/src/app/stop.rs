use tokio::sync::oneshot;
use yazi_shared::event::CmdCow;

pub struct StopOpt {
	pub tx: Option<oneshot::Sender<()>>,
}

impl From<CmdCow> for StopOpt {
	fn from(mut c: CmdCow) -> Self { Self { tx: c.take_any("tx") } }
}
