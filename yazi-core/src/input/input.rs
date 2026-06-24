use std::{ops::{Deref, DerefMut}, sync::Arc};

use parking_lot::Mutex;
use ratatui_widgets::block::Padding;
use yazi_binding::{elements::Spatial, position::Position};

use crate::input::{InputGuard, InputMutGuard};

#[derive(Default)]
pub struct Input {
	pub main: InputMain,
	pub alt:  Option<InputAlt>,
}

impl Input {
	pub fn focus(&self) -> bool { self.main.visible || self.alt.is_some() }

	pub fn padding(&self) -> Padding { Padding::new(1, 1, 1, 1) }

	pub fn position(&self) -> Option<Position> {
		if self.main.visible {
			Some(self.main.position)
		} else if let Some(alt) = &self.alt {
			Some(alt.position)
		} else {
			None
		}
	}

	pub fn lock(&self) -> Option<InputGuard<'_>> {
		if self.main.visible {
			Some(InputGuard::Main(&self.main.inner))
		} else if let Some(alt) = &self.alt {
			Some(InputGuard::Alt(alt.inner.lock()))
		} else {
			None
		}
	}

	pub fn lock_mut(&mut self) -> Option<InputMutGuard<'_>> {
		if self.main.visible {
			Some(InputMutGuard::Main(&mut self.main.inner))
		} else if let Some(alt) = &self.alt {
			Some(InputMutGuard::Alt(alt.inner.lock()))
		} else {
			None
		}
	}
}

// --- InputMain
#[derive(Default)]
pub struct InputMain {
	inner:        yazi_widgets::input::Input,
	pub id:       String,
	pub title:    String,
	pub position: Position,
	pub visible:  bool,
}

impl Deref for InputMain {
	type Target = yazi_widgets::input::Input;

	fn deref(&self) -> &Self::Target { &self.inner }
}

impl DerefMut for InputMain {
	fn deref_mut(&mut self) -> &mut Self::Target { &mut self.inner }
}

// --- InputAlt
pub struct InputAlt {
	inner:    Arc<Mutex<yazi_widgets::input::Input>>,
	position: Position,
}

impl Deref for InputAlt {
	type Target = Arc<Mutex<yazi_widgets::input::Input>>;

	fn deref(&self) -> &Self::Target { &self.inner }
}

impl From<&yazi_widgets::input::InputArc> for InputAlt {
	fn from(value: &yazi_widgets::input::InputArc) -> Self {
		Self { inner: value.deref().clone(), position: value.area().into() }
	}
}
