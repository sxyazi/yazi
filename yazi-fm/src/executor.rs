use yazi_config::{keymap::{Control, Key}, KEYMAP};
use yazi_core::input::InputMode;
use yazi_shared::{event::Exec, Layer};

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
			return cx.which.type_(key);
		}
		if cx.help.visible && cx.help.type_(&key) {
			return true;
		}
		if cx.input.visible && cx.input.type_(&key) {
			return true;
		}

		if cx.completion.visible {
			self.matches(Layer::Completion, key) || self.matches(Layer::Input, key)
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
		}
	}

	#[inline]
	fn matches(&mut self, layer: Layer, key: Key) -> bool {
		for Control { on, exec, .. } in KEYMAP.get(layer) {
			if on.is_empty() || on[0] != key {
				continue;
			}

			if on.len() > 1 {
				self.app.cx.which.show(&key, layer);
			} else {
				self.dispatch(exec, layer);
			}
			return true;
		}
		false
	}

	#[inline]
	pub(super) fn dispatch(&mut self, exec: &[Exec], layer: Layer) {
		for e in exec {
			match layer {
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
	}

	fn app(&mut self, exec: &Exec) {
		macro_rules! on {
			($name:ident) => {
				if exec.cmd == stringify!($name) {
					return self.app.$name(exec);
				}
			};
		}

		on!(plugin);
		on!(plugin_do);
		on!(stop);
	}

	fn manager(&mut self, exec: &Exec) {
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

		on!(MANAGER, update_files, &self.app.cx.tasks);
		on!(MANAGER, update_mimetype, &self.app.cx.tasks);
		on!(MANAGER, update_pages, &self.app.cx.tasks);
		on!(MANAGER, hover);
		on!(MANAGER, peek);
		on!(MANAGER, seek);
		on!(MANAGER, refresh, &self.app.cx.tasks);
		on!(MANAGER, quit, &self.app.cx.tasks);
		on!(MANAGER, close, &self.app.cx.tasks);
		on!(MANAGER, suspend);
		on!(ACTIVE, escape);
		on!(ACTIVE, preview);

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
		on!(MANAGER, open, &self.app.cx.tasks);
		on!(MANAGER, open_do, &self.app.cx.tasks);
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

		// Filter
		on!(ACTIVE, filter);
		on!(ACTIVE, filter_do);

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
			_ => {}
		}
	}

	fn tasks(&mut self, exec: &Exec) {
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
			_ => {}
		}
	}

	fn select(&mut self, exec: &Exec) {
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
			_ => {}
		}
	}

	fn input(&mut self, exec: &Exec) {
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
					_ => {}
				}
			}
			InputMode::Insert => {
				on!(backspace);
				on!(kill);
			}
		}
	}

	fn help(&mut self, exec: &Exec) {
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
			_ => {}
		}
	}

	fn completion(&mut self, exec: &Exec) {
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
			_ => {}
		}
	}
}
