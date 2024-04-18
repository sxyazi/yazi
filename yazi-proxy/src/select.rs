use tokio::sync::oneshot;
use yazi_config::popup::SelectCfg;
use yazi_shared::{emit, event::Cmd, Layer};

use crate::options::SelectOpt;

pub struct SelectProxy;

impl SelectProxy {
	#[inline]
	pub async fn show(cfg: SelectCfg) -> anyhow::Result<usize> {
		let (tx, rx) = oneshot::channel();
		emit!(Call(Cmd::new("show").with_any("option", SelectOpt { cfg, tx }), Layer::Select));
		rx.await?
	}
}
