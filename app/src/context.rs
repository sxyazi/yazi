use core::{input::Input, manager::Manager, select::Select, tasks::Tasks, which::Which, Position};

use config::keymap::KeymapLayer;
use libc::winsize;
use ratatui::prelude::Rect;
use shared::tty_size;

pub struct Ctx {
	pub manager: Manager,
	pub which:   Which,
	pub select:  Select,
	pub input:   Input,
	pub tasks:   Tasks,
}

impl Ctx {
	pub(super) fn new() -> Self {
		Self {
			manager: Manager::make(),
			which:   Default::default(),
			select:  Default::default(),
			input:   Default::default(),
			tasks:   Tasks::start(),
		}
	}

	pub(super) fn area(&self, pos: &Position) -> Rect {
		let winsize { ws_row, ws_col, .. } = tty_size();

		let (x, y) = match pos {
			Position::None => return Rect::default(),
			Position::Top(Rect { mut x, mut y, width, height }) => {
				x = x.min(ws_col.saturating_sub(*width));
				y = y.min(ws_row.saturating_sub(*height));
				((tty_size().ws_col / 2).saturating_sub(width / 2) + x, y)
			}
			Position::Hovered(rect @ Rect { mut x, y, width, height }) => {
				let Some(r) =
					self.manager.hovered().and_then(|h| self.manager.current().rect_current(&h.path))
				else {
					return self.area(&Position::Top(*rect));
				};

				x = x.min(ws_col.saturating_sub(*width));
				if y + height + r.y + r.height > ws_row {
					(x + r.x, r.y.saturating_sub(height.saturating_sub(1)))
				} else {
					(x + r.x, y + r.y + r.height)
				}
			}
		};

		let (w, h) = pos.dimension().unwrap();
		Rect { x, y, width: w.min(ws_col.saturating_sub(x)), height: h.min(ws_row.saturating_sub(y)) }
	}

	#[inline]
	pub(super) fn cursor(&self) -> Option<(u16, u16)> {
		if self.input.visible {
			let Rect { x, y, .. } = self.area(&self.input.position);
			return Some((x + 1 + self.input.cursor(), y + 1));
		}
		None
	}

	#[inline]
	pub(super) fn layer(&self) -> KeymapLayer {
		if self.which.visible {
			KeymapLayer::Which
		} else if self.input.visible {
			KeymapLayer::Input
		} else if self.select.visible {
			KeymapLayer::Select
		} else if self.tasks.visible {
			KeymapLayer::Tasks
		} else {
			KeymapLayer::Manager
		}
	}

	#[inline]
	pub(super) fn image_layer(&self) -> bool {
		!matches!(self.layer(), KeymapLayer::Which | KeymapLayer::Tasks)
	}
}
