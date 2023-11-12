use ratatui::prelude::Rect;

use crate::{Position, RectShim};

pub struct SelectOpt {
	pub title:    String,
	pub items:    Vec<String>,
	pub position: Position,
}

impl SelectOpt {
	pub fn top(title: &str, items: Vec<String>) -> Self {
		let height = 2 + items.len().min(/* TODO: hardcode */ 5) as u16;
		Self {
			title: title.to_owned(),
			items,
			position: Position::Top(
				// TODO:
				RectShim { x_offset: 0, y_offset: 2, width: 50, height },
			),
		}
	}

	pub fn hovered(title: &str, items: Vec<String>) -> Self {
		let height = 2 + items.len().min(/* TODO: hardcode */ 5) as u16;
		Self {
			title: title.to_owned(),
			items,
			position: Position::Hovered(
				// TODO:
				RectShim { x_offset: 0, y_offset: 1, width: 50, height },
			),
		}
	}
}
