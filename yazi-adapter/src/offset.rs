use ratatui::layout::Size;
use yazi_config::{PREVIEW, preview::{HorizontalAlignment, VerticalAlignment}};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Offset {
	pub x: u16,
	pub y: u16,
}

impl From<(Size, Size)> for Offset {
	fn from(value: (Size, Size)) -> Self {
		let inner = value.0;
		let outer = value.1;
		let offset_x = match PREVIEW.alignment.horizontal {
			HorizontalAlignment::Left => 0,
			HorizontalAlignment::Center => (outer.width - inner.width) / 2,
			HorizontalAlignment::Right => outer.width - inner.width,
		};
		let offset_y = match PREVIEW.alignment.vertical {
			VerticalAlignment::Top => 0,
			VerticalAlignment::Center => (outer.height - inner.height) / 2,
			VerticalAlignment::Bottom => outer.height - inner.height,
		};
		Self { x: offset_x, y: offset_y }
	}
}
