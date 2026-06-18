use std::sync::Arc;

use parking_lot::Mutex;
use yazi_binding::position::Position;

use crate::input::{InputGuard, InputMutGuard};

#[derive(Default)]
pub struct Input {
	pub main: yazi_widgets::input::Input,
	pub alt:  Option<Arc<Mutex<yazi_widgets::input::Input>>>,

	pub main_visible:  bool,
	pub main_title:    String,
	pub main_position: Position,
}

impl Input {
	pub fn focus(&self) -> bool { self.main_visible || self.alt.is_some() }

	pub fn lock(&self) -> Option<InputGuard<'_>> {
		if self.main_visible {
			Some(InputGuard::Main(&self.main))
		} else if let Some(alt) = &self.alt {
			Some(InputGuard::Alt(alt.lock()))
		} else {
			None
		}
	}

	pub fn lock_mut(&mut self) -> Option<InputMutGuard<'_>> {
		if self.main_visible {
			Some(InputMutGuard::Main(&mut self.main))
		} else if let Some(alt) = &self.alt {
			Some(InputMutGuard::Alt(alt.lock()))
		} else {
			None
		}
	}
}
