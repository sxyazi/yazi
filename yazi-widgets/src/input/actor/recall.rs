use anyhow::Result;
use yazi_macro::{render, succ};
use yazi_shared::data::Data;

use crate::{Step, input::{Input, InputSnap}};

impl Input {
	pub fn recall(&mut self, items: &[String], step: Step) -> Result<Data> {
		if items.is_empty() {
			succ!();
		}

		let pos = self.history.at.unwrap_or(items.len());
		let next = step.add(pos, items.len() + 1, 0, 0, 0);

		if next == pos {
			succ!();
		} else if next == items.len() {
			let draft = self.history.take();
			return self.recall_to(draft);
		} else if self.history.at.is_none() {
			self.history.draft = self.value().to_owned();
		}

		self.history.at = Some(next);
		self.recall_to(items[next].clone())
	}

	fn recall_to(&mut self, value: String) -> Result<Data> {
		let mode = self.mode();

		let mut snap = InputSnap::new(value, self.obscure);
		snap.mode = mode;
		snap.resize(self.size.width as usize);

		*self.snap_mut() = snap;
		self.flush_type();
		succ!(render!());
	}
}
