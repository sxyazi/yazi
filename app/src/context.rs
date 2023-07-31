use core::{input::Input, manager::Manager, select::Select, tasks::Tasks, which::Which, Position};

use config::keymap::KeymapLayer;
use shared::tty_size;

pub struct Ctx {
	pub manager: Manager,
	pub which:   Which,
	pub select:  Select,
	pub input:   Input,
	pub tasks:   Tasks,
}

impl Ctx {
	pub(super) fn new() -> Self {
		Self {
			manager: Manager::new(),
			which:   Default::default(),
			select:  Default::default(),
			input:   Default::default(),
			tasks:   Tasks::start(),
		}
	}

	#[inline]
	pub(super) fn cursor(&self) -> Option<(u16, u16)> {
		if self.input.visible {
			return Some(self.input.cursor());
		}
		None
	}

	#[inline]
	pub(super) fn layer(&self) -> KeymapLayer {
		if self.which.visible {
			KeymapLayer::Which
		} else if self.input.visible {
			KeymapLayer::Input
		} else if self.select.visible {
			KeymapLayer::Select
		} else if self.tasks.visible {
			KeymapLayer::Tasks
		} else {
			KeymapLayer::Manager
		}
	}

	#[inline]
	pub(super) fn image_layer(&self) -> bool {
		match self.layer() {
			KeymapLayer::Which => false,
			KeymapLayer::Tasks => false,
			_ => true,
		}
	}

	pub(super) fn position(&self, pos: Position) -> Position {
		match pos {
			Position::Top => Position::Coords((tty_size().ws_col / 2).saturating_sub(25), 2),
			Position::Hovered => self
				.manager
				.hovered()
				.and_then(|h| self.manager.current().rect_current(&h.path))
				.map(|r| Position::Coords(r.x, r.y))
				.unwrap_or_else(|| self.position(Position::Top)),
			p @ Position::Coords(..) => p,
		}
	}
}
