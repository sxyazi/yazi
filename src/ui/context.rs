use crate::{core::{input::Input, manager::Manager, select::Select, tasks::Tasks, Position}, misc::tty_size};

pub struct Ctx {
	pub cursor: Option<(u16, u16)>,

	pub manager: Manager,
	pub select:  Select,
	pub input:   Input,
	pub tasks:   Tasks,
}

impl Ctx {
	pub fn new() -> Self {
		Self {
			cursor: None,

			manager: Manager::new(),
			select:  Select::default(),
			input:   Input::default(),
			tasks:   Tasks::start(),
		}
	}

	pub fn position(&self, pos: Position) -> Position {
		match pos {
			Position::Top => Position::Coords((tty_size().ws_col / 2).saturating_sub(25), 2),
			Position::Hovered => self
				.manager
				.hovered()
				.and_then(|h| self.manager.current().rect_current(&h.path))
				.map(|r| Position::Coords(r.x, r.y))
				.unwrap_or_else(|| self.position(Position::Top)),
			p @ Position::Coords(..) => p,
		}
	}
}
