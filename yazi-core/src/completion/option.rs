use ratatui::layout::Rect;

use crate::Position;

pub struct CompletionOpt {
	pub items:    Vec<String>,
	pub position: Position,
}

impl CompletionOpt {
	pub fn top() -> Self {
		Self {
			items:    vec![],
			position: Position::Top(
				// TODO: hardcode
				Rect { x: 0, y: 5, width: 100, height: 10 },
			),
		}
	}

	pub fn hover() -> Self {
		Self {
			items:    vec![],
			position: Position::Hovered(
				// TODO: hardcode
				Rect { x: 0, y: 3, width: 40, height: 4 },
			),
		}
	}

	#[inline]
	pub fn with_items(mut self, items: Vec<String>) -> Self {
		self.items = items;
		self
	}
}
