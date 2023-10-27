use ratatui::layout::Rect;

use crate::Position;

pub struct CompletionOpt {
	pub items:      Vec<String>,
	pub position:   Position,
	pub column_cnt: u8,
	pub max_width:  u16,
}

impl CompletionOpt {
	pub fn top() -> Self {
		Self {
			items:      vec![],
			position:   Position::Top(
				// TODO: hardcode
				Rect { x: 0, y: 5, width: 80, height: 10 },
			),
			column_cnt: 4,
			max_width:  20,
		}
	}

	pub fn hover() -> Self {
		Self {
			items:      vec![],
			position:   Position::Hovered(
				// TODO: hardcode
				Rect { x: 0, y: 3, width: 80, height: 10 },
			),
			column_cnt: 4,
			max_width:  20,
		}
	}

	#[inline]
	pub fn with_items(mut self, items: Vec<String>) -> Self {
		self.items = items;
		self
	}
}
