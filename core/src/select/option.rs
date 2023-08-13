use ratatui::prelude::Rect;

use crate::Position;

pub struct SelectOpt {
	pub title:    String,
	pub items:    Vec<String>,
	pub position: Position,
}

impl SelectOpt {
	pub fn top(title: &str, items: Vec<String>) -> Self {
		Self {
			title: title.to_owned(),
			items,
			position: Position::Top(/* TODO: hardcode */ Rect { x: 0, y: 2, width: 50, height: 3 }),
		}
	}

	pub fn hovered(title: &str, items: Vec<String>) -> Self {
		Self {
			title: title.to_owned(),
			items,
			position: Position::Hovered(
				// TODO: hardcode
				Rect { x: 0, y: 1, width: 50, height: 3 },
			),
		}
	}
}
