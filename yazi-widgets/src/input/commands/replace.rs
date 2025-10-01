use anyhow::Result;
use yazi_macro::{render, succ};
use yazi_parser::VoidOpt;
use yazi_shared::{data::Data, replace_cow};

use crate::input::{Input, InputMode, op::InputOp};

impl Input {
	pub fn replace(&mut self, _: VoidOpt) -> Result<Data> {
		let snap = self.snap_mut();
		if snap.mode == InputMode::Normal {
			snap.op = InputOp::None;
			snap.mode = InputMode::Replace;
			render!();
		}
		succ!();
	}

	pub fn replace_str(&mut self, s: &str) -> Result<Data> {
		let s = replace_cow(replace_cow(s, "\r", " "), "\n", " ");

		let snap = self.snap_mut();
		snap.mode = InputMode::Normal;

		let start = snap.idx(snap.cursor).unwrap();
		let mut it = snap.value[start..].char_indices();
		match (it.next(), it.next()) {
			(None, _) => {}
			(Some(_), None) => snap.value.replace_range(start..snap.len(), &s),
			(Some(_), Some((len, _))) => snap.value.replace_range(start..start + len, &s),
		}

		self.snaps.tag(self.limit).then(|| self.flush_value());
		succ!(render!());
	}
}
