use tokio::sync::mpsc;
use yazi_config::popup::InputCfg;
use yazi_shared::{event::Cmd, InputError};

pub struct InputOpt {
	pub cfg: InputCfg,
	pub tx:  mpsc::UnboundedSender<Result<String, InputError>>,
}

impl TryFrom<Cmd> for InputOpt {
	type Error = ();

	fn try_from(mut c: Cmd) -> Result<Self, Self::Error> { c.take_data().ok_or(()) }
}
