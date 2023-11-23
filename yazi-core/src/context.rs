use ratatui::prelude::Rect;
use yazi_config::popup::{Origin, Position};

use crate::{completion::Completion, help::Help, input::Input, manager::Manager, select::Select, tasks::Tasks, which::Which};

pub struct Ctx {
	pub manager:    Manager,
	pub tasks:      Tasks,
	pub select:     Select,
	pub input:      Input,
	pub help:       Help,
	pub completion: Completion,
	pub which:      Which,
}

impl Ctx {
	pub fn make() -> Self {
		Self {
			manager:    Manager::make(),
			tasks:      Tasks::start(),
			select:     Default::default(),
			input:      Default::default(),
			help:       Default::default(),
			completion: Default::default(),
			which:      Default::default(),
		}
	}

	pub fn area(&self, position: &Position) -> Rect {
		if position.origin != Origin::Hovered {
			return position.rect();
		}

		if let Some(r) =
			self.manager.hovered().and_then(|h| self.manager.current().rect_current(&h.url))
		{
			Position::sticky(r, position.offset)
		} else {
			Position::new(Origin::TopCenter, position.offset).rect()
		}
	}

	#[inline]
	pub fn cursor(&self) -> Option<(u16, u16)> {
		if self.input.visible {
			let Rect { x, y, .. } = self.area(&self.input.position);
			return Some((x + 1 + self.input.cursor(), y + 1));
		}
		if let Some((x, y)) = self.help.cursor() {
			return Some((x, y));
		}
		None
	}
}
