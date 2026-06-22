use tokio::sync::mpsc;
use yazi_config::keymap::ChordArc;
use yazi_core::which::WhichOpt;
use yazi_macro::{emit, relay};
use yazi_shared::Layer;

pub struct WhichProxy;

impl WhichProxy {
	pub async fn activate(cands: Vec<ChordArc>, silent: bool) -> Option<ChordArc> {
		let (tx, mut rx) = mpsc::unbounded_channel();
		emit!(Call(relay!(which:activate).with_any("opt", WhichOpt {
			tx: Some(tx),
			layer: Layer::Null,
			cands,
			times: 0,
			silent,
		})));
		rx.recv().await?
	}
}
