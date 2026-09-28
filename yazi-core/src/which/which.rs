use tokio::sync::mpsc;
use yazi_config::keymap::{ChordArc, Key};
use yazi_macro::{emit, render_and};
use yazi_shared::Layer;

#[derive(Default)]
pub struct Which {
	pub tx:    Option<mpsc::UnboundedSender<Option<ChordArc>>>,
	pub layer: Layer,
	pub cands: Vec<ChordArc>,
	pub times: usize,

	// Active state
	pub active: bool,
	pub silent: bool,
}

impl Which {
	pub fn r#type(&mut self, key: Key) -> bool {
		if key.code.is_modifier() || key.code.is_lock() {
			return false;
		}

		self.cands.retain(|c| c.on.len() > self.times && c.on[self.times] == key);
		self.times += 1;

		// A label stays for the which component to name its group, but it neither counts as a
		// match nor ends the chord
		let mut runnable = self.cands.iter().enumerate().filter(|(_, c)| !c.is_label());
		let (first, more) = (runnable.next().map(|(i, _)| i), runnable.next().is_some());

		if first.is_none() {
			self.dismiss(None);
		} else if let (Some(i), false) = (first, more) {
			let chord = self.cands.remove(i);
			self.dismiss(Some(chord));
		} else if let Some(i) =
			self.cands.iter().position(|c| !c.is_label() && c.on.len() == self.times)
		{
			let chord = self.cands.remove(i);
			self.dismiss(Some(chord));
		}

		render_and!(true)
	}

	pub fn dismiss(&mut self, chord: Option<ChordArc>) {
		self.cands.clear();
		self.times = 0;

		self.active = false;
		self.silent = false;

		if let Some(tx) = self.tx.take() {
			_ = tx.send(chord.as_ref().map(Into::into));
		}
		if let Some(chord) = chord {
			emit!(Seq(chord.into_seq(self.layer)));
		}
	}
}
