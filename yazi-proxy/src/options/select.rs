use tokio::sync::oneshot;
use yazi_config::popup::SelectCfg;
use yazi_shared::event::Cmd;

pub struct SelectOpt {
	pub cfg: SelectCfg,
	pub tx:  oneshot::Sender<anyhow::Result<usize>>,
}

impl TryFrom<Cmd> for SelectOpt {
	type Error = ();

	fn try_from(mut c: Cmd) -> Result<Self, Self::Error> { c.take_data().ok_or(()) }
}
