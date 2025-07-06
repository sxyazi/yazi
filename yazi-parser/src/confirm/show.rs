use tokio::sync::oneshot;
use yazi_config::popup::ConfirmCfg;
use yazi_shared::event::CmdCow;

pub struct ShowOpt {
	pub cfg: ConfirmCfg,
	pub tx:  oneshot::Sender<bool>,
}

impl TryFrom<CmdCow> for ShowOpt {
	type Error = ();

	fn try_from(mut c: CmdCow) -> Result<Self, Self::Error> {
		Ok(Self { cfg: c.take_any("cfg").ok_or(())?, tx: c.take_any("tx").ok_or(())? })
	}
}
