use tokio::sync::mpsc;
use yazi_config::keymap::ChordCow;
use yazi_macro::{emit, relay};
use yazi_parser::which::ActivateOpt;

pub struct WhichProxy;

impl WhichProxy {
	pub async fn activate(cands: Vec<ChordCow>, silent: bool) -> Option<ChordCow> {
		let (tx, mut rx) = mpsc::unbounded_channel();
		emit!(Call(relay!(which:activate).with_any("opt", ActivateOpt {
			tx: Some(tx),
			cands,
			silent,
			times: 0,
		})));
		Some(rx.recv().await??.0)
	}
}
