use tokio::sync::oneshot;
use yazi_config::popup::PickCfg;
use yazi_macro::{emit, relay};

pub struct PickProxy;

impl PickProxy {
	pub async fn show(cfg: PickCfg) -> anyhow::Result<usize> {
		let (tx, rx) = oneshot::channel();
		emit!(Call(relay!(pick:show).with_any("tx", tx).with_any("cfg", cfg)));
		rx.await?
	}
}
