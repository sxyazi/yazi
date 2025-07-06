use tokio::sync::mpsc;
use yazi_config::popup::InputCfg;
use yazi_shared::{errors::InputError, event::CmdCow};

pub struct ShowOpt {
	pub cfg: InputCfg,
	pub tx:  mpsc::UnboundedSender<Result<String, InputError>>,
}

impl TryFrom<CmdCow> for ShowOpt {
	type Error = ();

	fn try_from(mut c: CmdCow) -> Result<Self, Self::Error> {
		Ok(Self { cfg: c.take_any("cfg").ok_or(())?, tx: c.take_any("tx").ok_or(())? })
	}
}
