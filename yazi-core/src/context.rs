use crossterm::terminal::WindowSize;
use ratatui::prelude::Rect;
use yazi_shared::Term;

use crate::{completion::Completion, help::Help, input::Input, manager::Manager, select::Select, tasks::Tasks, which::Which, Position};

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

		let (x, y) = match pos {
			Position::Top(Rect { mut x, mut y, width, height }) => {
				x = x.min(columns.saturating_sub(*width));
				y = y.min(rows.saturating_sub(*height));
				((columns / 2).saturating_sub(width / 2) + x, y)
			}
			Position::Sticky(Rect { mut x, y, width, height }, r) => {
				x = x.min(columns.saturating_sub(*width));
				if y + height + r.y + r.height > rows {
					(x + r.x, r.y.saturating_sub(height.saturating_sub(*y)))
				} else {
					(x + r.x, y + r.y + r.height)
				}
			}
			Position::Hovered(rect) => {
				return self.area(&if let Some(r) =
					self.manager.hovered().and_then(|h| self.manager.current().rect_current(&h.url))
				{
					Position::Sticky(*rect, r)
				} else {
					Position::Top(*rect)
				});
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
}
