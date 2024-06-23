use tokio::sync::oneshot;
use yazi_config::popup::ConfirmCfg;
use yazi_shared::{emit, event::Cmd, Layer};

pub struct ConfirmProxy;

impl ConfirmProxy {
	#[inline]
	pub async fn show(cfg: ConfirmCfg) -> bool {
		let (tx, rx) = oneshot::channel();
		emit!(Call(Cmd::new("show").with_any("tx", tx).with_any("cfg", cfg), Layer::Confirm));
		rx.await.unwrap_or(false)
	}
}
