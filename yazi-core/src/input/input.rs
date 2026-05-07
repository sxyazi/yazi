use std::ops::{Deref, DerefMut};

use anyhow::Result;
use yazi_config::popup::Position;
use yazi_macro::{render, succ};
use yazi_shared::{data::Data, event::ActionCow};
use yazi_widgets::input::{InputOp, parser::HistoryOpt};

#[derive(Default)]
pub struct Input {
	pub(super) inner: yazi_widgets::input::Input,
	pub history: std::collections::HashMap<String, yazi_widgets::input::InputHistory>,

	pub visible: bool,
	pub title: String,
	pub position: Position,
}

impl Input {
	pub fn execute(&mut self, action: ActionCow) -> Result<Data> {
		if action.name == "history" {
			return self.navigate_history(action.into());
		}
		self.inner.execute(action)
	}

	pub fn history(&mut self) -> &mut yazi_widgets::input::InputHistory {
		if !self.history.contains_key(&self.inner.id) {
			self.history.insert(self.inner.id.clone(), yazi_widgets::input::InputHistory::new());
		}
		self.history.get_mut(&self.inner.id).unwrap()
	}
	pub fn navigate_history(&mut self, opt: HistoryOpt) -> Result<Data> {
		if self.inner.snap().op != InputOp::None || self.inner.obscure {
			succ!();
		}
		match self.history.get_mut(&self.inner.id) {
			Some(history) => {
				if !history.navigate(opt.offset, &mut self.inner.snaps, self.inner.limit) {
					succ!();
				}
			}
			None => succ!(),
		}
		succ!(render!());
	}
}

impl Deref for Input {
	type Target = yazi_widgets::input::Input;

	fn deref(&self) -> &Self::Target {
		&self.inner
	}
}

impl DerefMut for Input {
	fn deref_mut(&mut self) -> &mut Self::Target {
		&mut self.inner
	}
}
