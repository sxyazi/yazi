use yazi_config::popup::{Offset as CfgOffset, Position as CfgPosition};

use crate::{Position, RectShim};

#[derive(Default)]
pub struct InputOpt {
	pub title:      String,
	pub value:      String,
	pub position:   Position,
	pub realtime:   bool,
	pub completion: bool,
	pub highlight:  bool,
}

macro_rules! gen_method {
	($func_name:ident, $position:ident) => {
		pub fn $func_name(title: impl AsRef<str>, rect: RectShim) -> InputOpt {
			InputOpt {
				title: title.as_ref().to_owned(),
				position: Position::$position(rect),
				..Default::default()
			}
		}
	};
}

impl InputOpt {
	gen_method!(top_left, TopLeft);

	gen_method!(top_right, TopRight);

	gen_method!(top_center, TopCenter);

	gen_method!(center, Center);

	gen_method!(bottom_center, BottomCenter);

	gen_method!(bottom_left, BottomLeft);

	gen_method!(bottom_right, BottomRight);

	gen_method!(hovered, Hovered);

	pub fn from_cfg(title: impl AsRef<str>, pos: &CfgPosition, rect: &CfgOffset) -> Self {
		let rect =
			RectShim { x_offset: rect.x, y_offset: rect.y, width: rect.width, height: rect.height };

		match pos {
			CfgPosition::TopLeft => Self::top_left(title, rect),
			CfgPosition::TopRight => Self::top_right(title, rect),
			CfgPosition::TopCenter => Self::top_center(title, rect),
			CfgPosition::Center => Self::center(title, rect),
			CfgPosition::BottomCenter => Self::bottom_center(title, rect),
			CfgPosition::BottomLeft => Self::bottom_left(title, rect),
			CfgPosition::BottomRight => Self::bottom_right(title, rect),
			CfgPosition::Hovered => Self::hovered(title, rect),
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
