use crossterm::terminal::WindowSize;
use ratatui::prelude::Rect;
use yazi_shared::Term;

use crate::{completion::Completion, help::Help, input::Input, manager::Manager, select::Select, tasks::Tasks, which::Which, Position, RectShim};

pub struct Ctx {
	pub manager:    Manager,
	pub tasks:      Tasks,
	pub select:     Select,
	pub input:      Input,
	pub help:       Help,
	pub completion: Completion,
	pub which:      Which,
}

impl Ctx {
	pub fn make() -> Self {
		Self {
			manager:    Manager::make(),
			tasks:      Tasks::start(),
			select:     Default::default(),
			input:      Default::default(),
			help:       Default::default(),
			completion: Default::default(),
			which:      Default::default(),
		}
	}

	pub fn area(&self, pos: &Position) -> Rect {
		let WindowSize { columns, rows, .. } = Term::size();

		let (x, y) = match *pos {
			Position::TopLeft(RectShim { x_offset, y_offset, width, height }) => {
				let right_max = columns.saturating_sub(width);
				let bottom_max = rows.saturating_sub(height);

				let base_x = 0_u16;
				let base_y = 0_u16;

				let x = base_x.saturating_add_signed(x_offset).max(0).min(right_max);
				let y = base_y.saturating_add_signed(y_offset).max(0).min(bottom_max);

				(x, y)
			}
			Position::TopRight(RectShim { x_offset, y_offset, width, height }) => {
				let right_max = columns.saturating_sub(width);
				let bottom_max = rows.saturating_sub(height);

				let base_x = right_max;
				let base_y = 0_u16;

				let x = base_x.saturating_add_signed(x_offset).max(0).min(right_max);
				let y = base_y.saturating_add_signed(y_offset).max(0).min(bottom_max);

				(x, y)
			}
			Position::TopCenter(RectShim { x_offset, y_offset, width, height }) => {
				let right_max = columns.saturating_sub(width);
				let bottom_max = rows.saturating_sub(height);

				let base_x = (columns / 2).saturating_sub(width / 2);
				let base_y = 0_u16;

				let x = base_x.saturating_add_signed(x_offset).max(0).min(right_max);
				let y = base_y.saturating_add_signed(y_offset).max(0).min(bottom_max);

				(x, y)
			}
			Position::Center(RectShim { x_offset, y_offset, width, height }) => {
				let right_max = columns.saturating_sub(width);
				let bottom_max = rows.saturating_sub(height);

				let base_x = (columns / 2).saturating_sub(width / 2);
				let base_y = (bottom_max / 2).saturating_sub(height / 2);

				let x = base_x.saturating_add_signed(x_offset).max(0).min(right_max);
				let y = base_y.saturating_add_signed(y_offset).max(0).min(bottom_max);

				(x, y)
			}
			Position::BottomCenter(RectShim { x_offset, y_offset, width, height }) => {
				let right_max = columns.saturating_sub(width);
				let bottom_max = rows.saturating_sub(height);

				let base_x = (columns / 2).saturating_sub(width / 2);
				let base_y = bottom_max;

				let x = base_x.saturating_add_signed(x_offset).max(0).min(right_max);
				let y = base_y.saturating_add_signed(y_offset).max(0).min(bottom_max);

				(x, y)
			}
			Position::BottomLeft(RectShim { x_offset, y_offset, width, height }) => {
				let right_max = columns.saturating_sub(width);
				let bottom_max = rows.saturating_sub(height);

				let base_x = 0_u16;
				let base_y = bottom_max;

				let x = base_x.saturating_add_signed(x_offset).max(0).min(right_max);
				let y = base_y.saturating_add_signed(y_offset).max(0).min(bottom_max);

				(x, y)
			}
			Position::BottomRight(RectShim { x_offset, y_offset, width, height }) => {
				let right_max = columns.saturating_sub(width);
				let bottom_max = rows.saturating_sub(height);

				let base_x = right_max;
				let base_y = bottom_max;

				let x = base_x.saturating_add_signed(x_offset).max(0).min(right_max);
				let y = base_y.saturating_add_signed(y_offset).max(0).min(bottom_max);

				(x, y)
			}
			Position::Hovered(rect_shim) => {
				return self.area(&if let Some(r) =
					self.manager.hovered().and_then(|h| self.manager.current().rect_current(&h.url))
				{
					Position::Sticky(rect_shim, r)
				} else {
					Position::TopCenter(rect_shim)
				});
			}
			Position::Sticky(RectShim { x_offset, y_offset, width, height }, r) => {
				// TODO:
				unimplemented!("Position::Sticky is not implemented");
				// let mut x = columns.saturating_sub(width);
				// x = x.saturating_add_signed(x_offset);
				// let mut y = r.y;
				// y = y.saturating_add_signed(y_offset);

				// if y + height + r.height > rows {
				// 	(x + r.x, r.y.saturating_sub(height.saturating_sub(y)))
				// } else {
				// 	(x + r.x, y + r.height)
				// }
			}
		};

		let (w, h) = pos.dimension();
		Rect { x, y, width: w.min(columns.saturating_sub(x)), height: h.min(rows.saturating_sub(y)) }
	}

	#[inline]
	pub fn cursor(&self) -> Option<(u16, u16)> {
		if self.input.visible {
			let Rect { x, y, .. } = self.area(&self.input.position);
			return Some((x + 1 + self.input.cursor(), y + 1));
		}
		if let Some((x, y)) = self.help.cursor() {
			return Some((x, y));
		}
		None
	}

	#[inline]
	pub fn image_layer(&self) -> bool {
		!self.which.visible && !self.help.visible && !self.tasks.visible
	}
}
