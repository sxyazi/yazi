use std::{collections::VecDeque, ops::Deref};

use yazi_shared::event::Cmd;

use super::Control;

#[derive(Debug)]
pub enum ControlCow {
	Owned(Control),
	Borrowed(&'static Control),
}

impl From<&'static Control> for ControlCow {
	fn from(c: &'static Control) -> Self { Self::Borrowed(c) }
}

impl From<Control> for ControlCow {
	fn from(c: Control) -> Self { Self::Owned(c) }
}

impl Deref for ControlCow {
	type Target = Control;

	fn deref(&self) -> &Self::Target {
		match self {
			Self::Owned(c) => c,
			Self::Borrowed(c) => c,
		}
	}
}

impl Default for ControlCow {
	fn default() -> Self { Self::Owned(Control::default()) }
}

impl ControlCow {
	pub fn into_seq(self) -> VecDeque<Cmd> {
		match self {
			Self::Owned(c) => c.run.into(),
			Self::Borrowed(c) => c.to_seq(),
		}
	}
}
