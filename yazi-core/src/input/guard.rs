use std::ops::{Deref, DerefMut};

use parking_lot::MutexGuard;

// --- InputGuard
pub enum InputGuard<'a> {
	Main(&'a yazi_widgets::input::Input),
	Alt(MutexGuard<'a, yazi_widgets::input::Input>),
}

impl Deref for InputGuard<'_> {
	type Target = yazi_widgets::input::Input;

	fn deref(&self) -> &Self::Target {
		match self {
			Self::Main(main) => main,
			Self::Alt(alt) => alt,
		}
	}
}

// --- InputMutGuard
pub enum InputMutGuard<'a> {
	Main(&'a mut yazi_widgets::input::Input),
	Alt(MutexGuard<'a, yazi_widgets::input::Input>),
}

impl Deref for InputMutGuard<'_> {
	type Target = yazi_widgets::input::Input;

	fn deref(&self) -> &Self::Target {
		match self {
			Self::Main(main) => main,
			Self::Alt(alt) => alt,
		}
	}
}

impl DerefMut for InputMutGuard<'_> {
	fn deref_mut(&mut self) -> &mut Self::Target {
		match self {
			Self::Main(main) => main,
			Self::Alt(alt) => alt,
		}
	}
}
