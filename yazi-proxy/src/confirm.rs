use tokio::sync::oneshot;
use yazi_config::popup::ConfirmCfg;
use yazi_macro::emit;
use yazi_shared::event::Cmd;

pub struct ConfirmProxy;

impl ConfirmProxy {
	#[inline]
	pub async fn show(cfg: ConfirmCfg) -> bool { Self::show_rx(cfg).await.unwrap_or(false) }

	#[inline]
	pub fn show_rx(cfg: ConfirmCfg) -> oneshot::Receiver<bool> {
		let (tx, rx) = oneshot::channel();
		emit!(Call(Cmd::new("confirm:show").with_any("tx", tx).with_any("cfg", cfg)));
		rx
	}
}
