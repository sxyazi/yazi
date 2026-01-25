use tokio::sync::mpsc;
use yazi_config::popup::PickCfg;
use yazi_macro::{emit, relay};

pub struct PickProxy;

impl PickProxy {
	pub async fn show(cfg: PickCfg) -> Option<usize> {
		let (tx, mut rx) = mpsc::unbounded_channel::<Option<usize>>();
		emit!(Call(relay!(pick:show).with_any("tx", tx).with_any("cfg", cfg)));
		rx.recv().await?
	}
}
