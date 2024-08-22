use ratatui::layout::Rect;
use yazi_core::{completion::Completion, confirm::Confirm, help::Help, input::Input, manager::Manager, notify::Notify, select::Select, tasks::Tasks, which::Which};

pub struct Ctx {
	pub manager:    Manager,
	pub tasks:      Tasks,
	pub select:     Select,
	pub input:      Input,
	pub confirm:    Confirm,
	pub help:       Help,
	pub completion: Completion,
	pub which:      Which,
	pub notify:     Notify,
}

impl Ctx {
	pub fn make() -> Self {
		Self {
			manager:    Manager::make(),
			tasks:      Tasks::serve(),
			select:     Default::default(),
			input:      Default::default(),
			confirm:    Default::default(),
			help:       Default::default(),
			completion: Default::default(),
			which:      Default::default(),
			notify:     Default::default(),
		}
	}

	#[inline]
	pub fn cursor(&self) -> Option<(u16, u16)> {
		if self.input.visible {
			let Rect { x, y, .. } = self.manager.area(self.input.position);
			return Some((x + 1 + self.input.cursor(), y + 1));
		}
		if let Some((x, y)) = self.help.cursor() {
			return Some((x, y));
		}
		None
	}
}
