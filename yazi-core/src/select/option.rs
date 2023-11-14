use yazi_config::popup::{Offset as CfgOffset, Position as CfgPosition};

use crate::{Position, Offset};

pub struct SelectOpt {
	pub title:    String,
	pub items:    Vec<String>,
	pub position: Position,
}

macro_rules! gen_method {
	($func_name:ident, $position:ident) => {
		pub fn $func_name(title: &str, items: Vec<String>, rect: Offset) -> SelectOpt {
			let height = 2
				+ items.len().min(
					5, // TODO: hardcode
				) as u16;
			Self {
				title: title.to_owned(),
				items,
				position: Position::$position(Offset { height, ..rect }),
			}
		}
	};
}

impl SelectOpt {
	gen_method!(top_left, TopLeft);

	gen_method!(top_right, TopRight);

	gen_method!(top_center, TopCenter);

	gen_method!(center, Center);

	gen_method!(bottom_center, BottomCenter);

	gen_method!(bottom_left, BottomLeft);

	gen_method!(bottom_right, BottomRight);

	gen_method!(hovered, Hovered);

	pub fn from_cfg(title: &str, items: Vec<String>, pos: &CfgPosition, rect: &CfgOffset) -> Self {
		let rect =
			Offset { x_offset: rect.x, y_offset: rect.y, width: rect.width, height: rect.height };

		match pos {
			CfgPosition::TopLeft => Self::top_left(title, items, rect),
			CfgPosition::TopRight => Self::top_right(title, items, rect),
			CfgPosition::TopCenter => Self::top_center(title, items, rect),
			CfgPosition::Center => Self::center(title, items, rect),
			CfgPosition::BottomCenter => Self::bottom_center(title, items, rect),
			CfgPosition::BottomLeft => Self::bottom_left(title, items, rect),
			CfgPosition::BottomRight => Self::bottom_right(title, items, rect),
			CfgPosition::Hovered => Self::hovered(title, items, rect),
		}
	}
}
