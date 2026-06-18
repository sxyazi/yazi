use tokio::sync::mpsc;
use yazi_config::keymap::ChordArc;
use yazi_core::which::WhichOpt;
use yazi_macro::{emit, relay};

pub struct WhichProxy;

impl WhichProxy {
	pub async fn activate(cands: Vec<ChordArc>, silent: bool) -> Option<ChordArc> {
		let (tx, mut rx) = mpsc::unbounded_channel();
		emit!(Call(relay!(which:activate).with_any("opt", WhichOpt {
			tx: Some(tx),
			cands,
			silent,
			times: 0,
		})));
		rx.recv().await?
	}
}
