use yazi_config::{keymap::{Control, Key}, KEYMAP};
use yazi_core::input::InputMode;
use yazi_shared::{Exec, Layer};

use crate::app::App;

pub(super) struct Executor<'a> {
	app: &'a mut App,
}

impl<'a> Executor<'a> {
	#[inline]
	pub(super) fn new(app: &'a mut App) -> Self { Self { app } }

	pub(super) fn handle(&mut self, key: Key) -> bool {
		let cx = &mut self.app.cx;

		if cx.which.visible {
			return cx.which.press(key);
		}
		if cx.help.visible && cx.help.type_(&key) {
			return true;
		}
		if cx.input.visible && cx.input.type_(&key) {
			return true;
		}

		let b = if cx.completion.visible {
			self.matches(Layer::Completion, key).or_else(|| self.matches(Layer::Input, key))
		} else if cx.help.visible {
			self.matches(Layer::Help, key)
		} else if cx.input.visible {
			self.matches(Layer::Input, key)
		} else if cx.select.visible {
			self.matches(Layer::Select, key)
		} else if cx.tasks.visible {
			self.matches(Layer::Tasks, key)
		} else {
			self.matches(Layer::Manager, key)
		};
		b == Some(true)
	}

	#[inline]
	fn matches(&mut self, layer: Layer, key: Key) -> Option<bool> {
		for Control { on, exec, .. } in KEYMAP.get(layer) {
			if on.is_empty() || on[0] != key {
				continue;
			}

			return Some(if on.len() > 1 {
				self.app.cx.which.show(&key, layer)
			} else {
				self.dispatch(exec, layer)
			});
		}
		None
	}

	#[inline]
	pub(super) fn dispatch(&mut self, exec: &[Exec], layer: Layer) -> bool {
		let mut render = false;
		for e in exec {
			render |= match layer {
				Layer::App => self.app(e),
				Layer::Manager => self.manager(e),
				Layer::Tasks => self.tasks(e),
				Layer::Select => self.select(e),
				Layer::Input => self.input(e),
				Layer::Help => self.help(e),
				Layer::Completion => self.completion(e),
				Layer::Which => unreachable!(),
			};
		}
		render
	}

	fn app(&mut self, exec: &Exec) -> bool {
		macro_rules! on {
			($name:ident) => {
				if exec.cmd == stringify!($name) {
					return self.app.$name(exec);
				}
			};
		}

		on!(stop);

		false
	}

	fn manager(&mut self, exec: &Exec) -> bool {
		macro_rules! on {
			(MANAGER, $name:ident $(,$args:expr)*) => {
				if exec.cmd == stringify!($name) {
					return self.app.cx.manager.$name(exec, $($args),*);
				}
			};
			(ACTIVE, $name:ident) => {
				if exec.cmd == stringify!($name) {
					return self.app.cx.manager.active_mut().$name(exec);
				}
			};
			(TABS, $name:ident) => {
				if exec.cmd == concat!("tab_", stringify!($name)) {
					return self.app.cx.manager.tabs.$name(exec);
				}
			};
		}

		on!(MANAGER, peek);
		on!(MANAGER, hover);
		on!(MANAGER, refresh);
		on!(MANAGER, quit, &self.app.cx.tasks);
		on!(MANAGER, close, &self.app.cx.tasks);
		on!(MANAGER, suspend);
		on!(ACTIVE, escape);

		// Navigation
		on!(ACTIVE, arrow);
		on!(ACTIVE, leave);
		on!(ACTIVE, enter);
		on!(ACTIVE, back);
		on!(ACTIVE, forward);
		on!(ACTIVE, cd);
		on!(ACTIVE, reveal);

		// Selection
		on!(ACTIVE, select);
		on!(ACTIVE, select_all);
		on!(ACTIVE, visual_mode);

		// Operation
		on!(MANAGER, open);
		on!(MANAGER, yank);
		on!(MANAGER, paste, &self.app.cx.tasks);
		on!(MANAGER, link, &self.app.cx.tasks);
		on!(MANAGER, remove, &self.app.cx.tasks);
		on!(MANAGER, create);
		on!(MANAGER, rename);
		on!(ACTIVE, copy);
		on!(ACTIVE, shell);
		on!(ACTIVE, hidden);
		on!(ACTIVE, linemode);
		on!(ACTIVE, search);
		on!(ACTIVE, jump);

		// Find
		on!(ACTIVE, find);
		on!(ACTIVE, find_do);
		on!(ACTIVE, find_arrow);

		// Sorting
		on!(ACTIVE, sort);

		// Tabs
		on!(TABS, create);
		on!(TABS, close);
		on!(TABS, switch);
		on!(TABS, swap);

		match exec.cmd.as_bytes() {
			// Tasks
			b"tasks_show" => self.app.cx.tasks.toggle(()),
			// Help
			b"help" => self.app.cx.help.toggle(Layer::Manager),
			_ => false,
		}
	}

	fn tasks(&mut self, exec: &Exec) -> bool {
		macro_rules! on {
			($name:ident) => {
				if exec.cmd == stringify!($name) {
					return self.app.cx.tasks.$name(exec);
				}
			};
			($name:ident, $alias:literal) => {
				if exec.cmd == $alias {
					return self.app.cx.tasks.$name(exec);
				}
			};
		}

		on!(update);
		on!(open);
		on!(toggle, "close");
		on!(arrow);
		on!(inspect);
		on!(cancel);

		match exec.cmd.as_str() {
			"help" => self.app.cx.help.toggle(Layer::Tasks),
			_ => false,
		}
	}

	fn select(&mut self, exec: &Exec) -> bool {
		macro_rules! on {
			($name:ident) => {
				if exec.cmd == stringify!($name) {
					return self.app.cx.select.$name(exec);
				}
			};
		}

		on!(show);
		on!(close);
		on!(arrow);

		match exec.cmd.as_str() {
			"help" => self.app.cx.help.toggle(Layer::Select),
			_ => false,
		}
	}

	fn input(&mut self, exec: &Exec) -> bool {
		macro_rules! on {
			($name:ident) => {
				if exec.cmd == stringify!($name) {
					return self.app.cx.input.$name(exec);
				}
			};
			($name:ident, $alias:literal) => {
				if exec.cmd == $alias {
					return self.app.cx.input.$name(exec);
				}
			};
		}

		on!(show);
		on!(close);
		on!(escape);
		on!(move_, "move");
		on!(backward);
		on!(forward);

		if exec.cmd.as_str() == "complete" {
			return if exec.named.contains_key("trigger") {
				self.app.cx.completion.trigger(exec)
			} else {
				self.app.cx.input.complete(exec)
			};
		}

		match self.app.cx.input.mode() {
			InputMode::Normal => {
				on!(insert);
				on!(visual);

				on!(delete);
				on!(yank);
				on!(paste);

				on!(undo);
				on!(redo);

				match exec.cmd.as_str() {
					"help" => self.app.cx.help.toggle(Layer::Input),
					_ => false,
				}
			}
			InputMode::Insert => {
				on!(backspace);
				on!(kill);

				false
			}
		}
	}

	fn help(&mut self, exec: &Exec) -> bool {
		macro_rules! on {
			($name:ident) => {
				if exec.cmd == stringify!($name) {
					return self.app.cx.help.$name(exec);
				}
			};
		}

		on!(escape);
		on!(arrow);
		on!(filter);

		match exec.cmd.as_str() {
			"close" => self.app.cx.help.toggle(Layer::Help),
			_ => false,
		}
	}

	fn completion(&mut self, exec: &Exec) -> bool {
		macro_rules! on {
			($name:ident) => {
				if exec.cmd == stringify!($name) {
					return self.app.cx.completion.$name(exec);
				}
			};
		}

		on!(trigger);
		on!(show);
		on!(close);
		on!(arrow);

		match exec.cmd.as_str() {
			"help" => self.app.cx.help.toggle(Layer::Completion),
			_ => false,
		}
	}
}
