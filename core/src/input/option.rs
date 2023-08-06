use crate::Position;

pub struct InputOpt {
	pub title:     String,
	pub value:     String,
	pub position:  Position,
	pub highlight: bool,
}

impl InputOpt {
	pub fn top(title: impl AsRef<str>) -> Self {
		Self {
			title:     title.as_ref().to_owned(),
			value:     String::new(),
			position:  Position::Top,
			highlight: false,
		}
	}

	pub fn hovered(title: impl AsRef<str>) -> Self {
		Self {
			title:     title.as_ref().to_owned(),
			value:     String::new(),
			position:  Position::Hovered,
			highlight: false,
		}
	}

	pub fn with_value(mut self, value: impl AsRef<str>) -> Self {
		self.value = value.as_ref().to_owned();
		self
	}

	pub fn with_highlight(mut self) -> Self {
		self.highlight = true;
		self
	}
}
