use tokio::sync::oneshot;
use yazi_config::popup::SelectCfg;
use yazi_shared::{emit, event::Cmd, Layer};

pub struct SelectProxy;

impl SelectProxy {
	#[inline]
	pub async fn show(cfg: SelectCfg) -> anyhow::Result<usize> {
		let (tx, rx) = oneshot::channel();
		emit!(Call(Cmd::new("show").with_any("tx", tx).with_any("cfg", cfg), Layer::Select));
		rx.await?
	}
}
