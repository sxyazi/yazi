use tokio::sync::oneshot;
use yazi_config::popup::PickCfg;
use yazi_macro::emit;
use yazi_shared::event::Cmd;

pub struct PickProxy;

impl PickProxy {
	#[inline]
	pub async fn show(cfg: PickCfg) -> anyhow::Result<usize> {
		let (tx, rx) = oneshot::channel();
		emit!(Call(Cmd::new("pick:show").with_any("tx", tx).with_any("cfg", cfg)));
		rx.await?
	}
}
