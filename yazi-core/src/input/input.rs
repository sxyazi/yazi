use std::ops::{Deref, DerefMut};

use anyhow::Result;
use yazi_config::popup::Position;
use yazi_shared::{data::Data, event::ActionCow};

#[derive(Default)]
pub struct Input {
	pub(super) inner: yazi_widgets::input::Input,
	pub history:      yazi_widgets::input::InputHistory,

	pub visible:  bool,
	pub title:    String,
	pub position: Position,
}

impl Input {
	pub fn execute(&mut self, action: ActionCow) -> Result<Data> {
		self.inner.execute(action, &mut self.history)
	}
}

impl Deref for Input {
	type Target = yazi_widgets::input::Input;

	fn deref(&self) -> &Self::Target { &self.inner }
}

impl DerefMut for Input {
	fn deref_mut(&mut self) -> &mut Self::Target { &mut self.inner }
}
