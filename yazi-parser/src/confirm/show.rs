use anyhow::bail;
use tokio::sync::oneshot;
use yazi_config::popup::ConfirmCfg;
use yazi_shared::event::CmdCow;

pub struct ShowOpt {
	pub cfg: ConfirmCfg,
	pub tx:  oneshot::Sender<bool>,
}

impl TryFrom<CmdCow> for ShowOpt {
	type Error = anyhow::Error;

	fn try_from(mut c: CmdCow) -> Result<Self, Self::Error> {
		let Some(cfg) = c.take_any("cfg") else {
			bail!("Invalid 'cfg' argument in ShowOpt");
		};

		let Some(tx) = c.take_any("tx") else {
			bail!("Invalid 'tx' argument in ShowOpt");
		};

		Ok(Self { cfg, tx })
	}
}
