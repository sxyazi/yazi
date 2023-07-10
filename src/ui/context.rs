use crate::core::{input::Input, manager::Manager, tasks::Tasks};

pub struct Ctx {
	pub cursor: Option<(u16, u16)>,

	pub manager: Manager,
	pub input:   Input,
	pub tasks:   Tasks,
}

impl Ctx {
	pub fn new() -> Self {
		Self {
			cursor: None,

			manager: Manager::new(),
			input:   Input::default(),
			tasks:   Tasks::start(),
		}
	}
}
