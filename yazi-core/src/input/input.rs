use std::{ops::{Deref, DerefMut}, sync::Arc};

use parking_lot::Mutex;
use ratatui_core::layout::Rect;
use ratatui_widgets::block::Padding;
use yazi_binding::{elements::Spatial, position::Position};
use yazi_config::LAYOUT;
use yazi_shared::{id::Id, url::UrlBuf};
use yazi_shim::ratatui::Padable;

use crate::{input::{InputGuard, InputHistories, InputMutGuard}, mgr::Mgr};

#[derive(Default)]
pub struct Input {
	pub main:      InputMain,
	pub alt:       Option<InputAlt>,
	pub histories: InputHistories,
}

impl Input {
	pub(crate) fn focus(&self) -> bool { self.main.visible || self.alt.is_some() }

	pub fn padding(&self) -> Padding { Padding::new(1, 1, 1, 1) }

	pub fn main_field_area(&self, mgr: &Mgr) -> Rect {
		if let Some(area) = self.main.command_area() {
			area.padding(Padding::left(1))
		} else {
			mgr.area(self.main.position).padding(self.padding())
		}
	}

	pub(crate) fn position(&self) -> Option<Position> {
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
			Some(InputMutGuard::Main(self))
		} else if let Some(alt) = &mut self.alt {
			let guard = alt.inner.lock_arc();
			Some(InputMutGuard::Alt(self, guard))
		} else {
			None
		}
	}
}

// --- InputMain
#[derive(Default)]
pub struct InputMain {
	pub(super) inner:   yazi_widgets::input::Input,
	pub name:           String,
	pub title:          String,
	pub position:       Position,
	pub visible:        bool,
	pub command_target: Option<(Id, UrlBuf)>,
}

impl InputMain {
	pub fn is_view_command(&self) -> bool { self.command_target.is_some() }

	pub fn command_area(&self) -> Option<Rect> {
		if !self.is_view_command() {
			return None;
		}
		let area = LAYOUT.get().view;
		Some(Rect::new(area.x, area.bottom().saturating_sub(1), area.width, area.height.min(1)))
	}
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
