use ratatui::prelude::Rect;

use crate::Position;

#[derive(Default)]
pub struct InputOpt {
	pub title:      String,
	pub value:      String,
	pub position:   Position,
	pub realtime:   bool,
	pub completion: bool,
	pub highlight:  bool,
}

impl InputOpt {
	pub fn top(title: impl AsRef<str>) -> Self {
		Self {
			title: title.as_ref().to_owned(),
			position: Position::Top(/* TODO: hardcode */ Rect { x: 0, y: 2, width: 50, height: 3 }),
			..Default::default()
		}
	}

	pub fn hovered(title: impl AsRef<str>) -> Self {
		Self {
			title: title.as_ref().to_owned(),
			position: Position::Hovered(
				// TODO: hardcode
				Rect { x: 0, y: 1, width: 50, height: 3 },
			),
			..Default::default()
		}
	}

	#[inline]
	pub fn with_value(mut self, value: impl AsRef<str>) -> Self {
		self.value = value.as_ref().to_owned();
		self
	}

	#[inline]
	pub fn with_realtime(mut self) -> Self {
		self.realtime = true;
		self
	}

	#[inline]
	pub fn with_completion(mut self) -> Self {
		self.completion = true;
		self
	}

	#[inline]
	pub fn with_highlight(mut self) -> Self {
		self.highlight = true;
		self
	}
}
