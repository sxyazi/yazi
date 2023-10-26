use ratatui::layout::Rect;

use crate::Position;

pub struct CompletionOpt {
	pub items:    Vec<String>,
	pub position: Position,
}

impl CompletionOpt {
	pub fn hovered() -> Self {
		Self {
			items:    vec![],
			position: Position::Hovered(
				// TODO: hardcode
				Rect { x: 0, y: 1, width: 50, height: 3 },
			),
		}
	}

	#[inline]
	pub fn with_items(mut self, items: Vec<String>) -> Self {
		self.items = items;
		self
	}
}
