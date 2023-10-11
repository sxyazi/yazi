use config::keymap::KeymapLayer;
use crossterm::terminal::WindowSize;
use ratatui::prelude::Rect;
use shared::Term;

use crate::{help::Help, input::Input, manager::Manager, select::Select, tasks::Tasks, which::Which, Position};

pub struct Ctx {
	pub manager: Manager,
	pub which:   Which,
	pub help:    Help,
	pub input:   Input,
	pub select:  Select,
	pub tasks:   Tasks,
}

impl Ctx {
	pub fn make() -> Self {
		Self {
			manager: Manager::make(),
			which:   Default::default(),
			help:    Default::default(),
			input:   Default::default(),
			select:  Default::default(),
			tasks:   Tasks::start(),
		}
	}

	pub fn area(&self, pos: &Position) -> Rect {
		let WindowSize { columns, rows, .. } = Term::size();

		let (x, y) = match pos {
			Position::None => return Rect::default(),
			Position::Top(Rect { mut x, mut y, width, height }) => {
				x = x.min(columns.saturating_sub(*width));
				y = y.min(rows.saturating_sub(*height));
				((columns / 2).saturating_sub(width / 2) + x, y)
			}
			Position::Hovered(rect @ Rect { mut x, y, width, height }) => {
				let Some(r) =
					self.manager.hovered().and_then(|h| self.manager.current().rect_current(h.url()))
				else {
					return self.area(&Position::Top(*rect));
				};

				x = x.min(columns.saturating_sub(*width));
				if y + height + r.y + r.height > rows {
					(x + r.x, r.y.saturating_sub(height.saturating_sub(1)))
				} else {
					(x + r.x, y + r.y + r.height)
				}
			}
		};

		let (w, h) = pos.dimension().unwrap();
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
	pub fn layer(&self) -> KeymapLayer {
		if self.which.visible {
			KeymapLayer::Which
		} else if self.help.visible() {
			KeymapLayer::Help
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
	pub fn image_layer(&self) -> bool {
		!matches!(self.layer(), KeymapLayer::Which | KeymapLayer::Help | KeymapLayer::Tasks)
	}
}
