use tokio::sync::oneshot;
use yazi_config::popup::PickCfg;
use yazi_shared::event::CmdCow;

pub struct ShowOpt {
	pub cfg: PickCfg,
	pub tx:  oneshot::Sender<anyhow::Result<usize>>,
}

impl TryFrom<CmdCow> for ShowOpt {
	type Error = ();

	fn try_from(mut c: CmdCow) -> Result<Self, Self::Error> {
		Ok(Self { cfg: c.take_any("cfg").ok_or(())?, tx: c.take_any("tx").ok_or(())? })
	}
}
