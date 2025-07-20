use tokio::sync::oneshot;
use yazi_config::popup::ConfirmCfg;
use yazi_macro::{emit, relay};

pub struct ConfirmProxy;

impl ConfirmProxy {
	pub async fn show(cfg: ConfirmCfg) -> bool { Self::show_rx(cfg).await.unwrap_or(false) }

	pub fn show_rx(cfg: ConfirmCfg) -> oneshot::Receiver<bool> {
		let (tx, rx) = oneshot::channel();
		emit!(Call(relay!(confirm:show).with_any("tx", tx).with_any("cfg", cfg)));
		rx
	}
}
