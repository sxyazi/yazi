use std::{ops::{Deref, DerefMut}, rc::Rc};

use tokio::sync::mpsc::UnboundedSender;
use yazi_config::popup::Position;
use yazi_shared::{Ids, errors::InputError};

#[derive(Default)]
pub struct Input {
	pub(super) inner: yazi_widgets::input::Input,

	pub visible:  bool,
	pub title:    String,
	pub position: Position,

	// Typing
	pub(super) tx:     Option<UnboundedSender<Result<String, InputError>>>,
	pub(super) ticket: Rc<Ids>,
}

impl Deref for Input {
	type Target = yazi_widgets::input::Input;

	fn deref(&self) -> &Self::Target { &self.inner }
}

impl DerefMut for Input {
	fn deref_mut(&mut self) -> &mut Self::Target { &mut self.inner }
}
