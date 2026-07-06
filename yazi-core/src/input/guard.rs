use std::ops::{Deref, DerefMut};

use parking_lot::{ArcMutexGuard, MutexGuard, RawMutex};

use crate::input::Input;

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
	Main(&'a mut Input),
	Alt(&'a mut Input, ArcMutexGuard<RawMutex, yazi_widgets::input::Input>),
}

impl Deref for InputMutGuard<'_> {
	type Target = yazi_widgets::input::Input;

	fn deref(&self) -> &Self::Target {
		match self {
			Self::Main(input) => &input.main.inner,
			Self::Alt(_, guard) => guard,
		}
	}
}

impl DerefMut for InputMutGuard<'_> {
	fn deref_mut(&mut self) -> &mut Self::Target {
		match self {
			Self::Main(input) => &mut input.main.inner,
			Self::Alt(_, guard) => guard,
		}
	}
}
