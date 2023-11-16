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

	pub fn area(&self, pos: &Position) -> Rect {
		// TODO: hovered
		if let Origin::Hovered = pos.origin {
			return Rect::default();
		}

		pos.rect()
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

	#[inline]
	pub fn image_layer(&self) -> bool {
		!self.which.visible && !self.help.visible && !self.tasks.visible
	}
}
