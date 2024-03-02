use tokio::sync::oneshot;
use yazi_config::popup::SelectCfg;
use yazi_shared::{emit, event::Cmd, term::Term, Layer};

pub struct SelectOpt {
	pub cfg: SelectCfg,
	pub tx:  oneshot::Sender<anyhow::Result<usize>>,
}

impl TryFrom<Cmd> for SelectOpt {
	type Error = ();

	fn try_from(mut c: Cmd) -> Result<Self, Self::Error> { c.take_data().ok_or(()) }
}

pub struct SelectProxy;

impl SelectProxy {
	#[inline]
	pub async fn show(cfg: SelectCfg) -> anyhow::Result<usize> {
		let (tx, rx) = oneshot::channel();
		emit!(Call(Cmd::new("show").with_data(SelectOpt { cfg, tx }), Layer::Select));
		rx.await.unwrap_or_else(|_| Term::goodbye(|| false))
	}
}
