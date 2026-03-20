use std::ops::{Deref, DerefMut};

use yazi_config::popup::Position;

#[derive(Default)]
pub struct Input {
	pub(super) inner: yazi_widgets::input::Input,

	pub visible:  bool,
	pub title:    String,
	pub position: Position,
}

impl Deref for Input {
	type Target = yazi_widgets::input::Input;

	fn deref(&self) -> &Self::Target { &self.inner }
}

impl DerefMut for Input {
	fn deref_mut(&mut self) -> &mut Self::Target { &mut self.inner }
}
