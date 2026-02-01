use tokio::sync::mpsc;
use yazi_config::keymap::{ChordCow, Key};
use yazi_macro::{emit, render_and};

#[derive(Default)]
pub struct Which {
	pub tx:    Option<mpsc::UnboundedSender<Option<yazi_binding::ChordCow>>>,
	pub times: usize,
	pub cands: Vec<ChordCow>,

	// Active state
	pub active: bool,
	pub silent: bool,
}

impl Which {
	pub fn r#type(&mut self, key: Key) -> bool {
		self.cands.retain(|c| c.on.len() > self.times && c.on[self.times] == key);
		self.times += 1;

		if self.cands.is_empty() {
			self.dismiss(None);
		} else if self.cands.len() == 1 {
			let chord = self.cands.remove(0);
			self.dismiss(Some(chord));
		} else if let Some(i) = self.cands.iter().position(|c| c.on.len() == self.times) {
			let chord = self.cands.remove(i);
			self.dismiss(Some(chord));
		}

		render_and!(true)
	}

	pub fn dismiss(&mut self, chord: Option<ChordCow>) {
		self.times = 0;
		self.cands.clear();

		self.active = false;
		self.silent = false;

		if let Some(tx) = self.tx.take() {
			_ = tx.send(chord.clone().map(Into::into));
		}
		if let Some(chord) = chord {
			emit!(Seq(chord.into_seq()));
		}
	}
}
