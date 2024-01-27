use yazi_core::input::InputMode;
use yazi_shared::{event::Exec, Layer};

use crate::app::App;

pub(super) struct Executor<'a> {
	app: &'a mut App,
}

impl<'a> Executor<'a> {
	#[inline]
	pub(super) fn new(app: &'a mut App) -> Self { Self { app } }

	#[inline]
	pub(super) fn execute(&mut self, exec: Exec, layer: Layer) {
		match layer {
			Layer::App => self.app(exec),
			Layer::Manager => self.manager(exec),
			Layer::Tasks => self.tasks(exec),
			Layer::Select => self.select(exec),
			Layer::Input => self.input(exec),
			Layer::Help => self.help(exec),
			Layer::Completion => self.completion(exec),
			Layer::Which => unreachable!(),
		}
	}

	fn app(&mut self, exec: Exec) {
		macro_rules! on {
			($name:ident) => {
				if exec.cmd == stringify!($name) {
					return self.app.$name(exec);
				}
			};
		}

		on!(plugin);
		on!(plugin_do);
		on!(update_progress);
		on!(resize);
		on!(stop);
		on!(resume);
	}

	fn manager(&mut self, exec: Exec) {
		macro_rules! on {
			(MANAGER, $name:ident $(,$args:expr)*) => {
				if exec.cmd == stringify!($name) {
					return self.app.cx.manager.$name(exec, $($args),*);
				}
			};
			(ACTIVE, $name:ident $(,$args:expr)*) => {
				if exec.cmd == stringify!($name) {
					return self.app.cx.manager.active_mut().$name(exec, $($args),*);
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
		on!(MANAGER, update_paged, &self.app.cx.tasks);
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
		on!(ACTIVE, sort, &self.app.cx.tasks);

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
			// Plugin
			b"plugin" => self.app.plugin(exec),
			_ => {}
		}
	}

	fn tasks(&mut self, exec: Exec) {
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

		on!(open);
		on!(toggle, "close");
		on!(arrow);
		on!(inspect);
		on!(cancel);

		#[allow(clippy::single_match)]
		match exec.cmd.as_str() {
			"help" => self.app.cx.help.toggle(Layer::Tasks),
			_ => {}
		}
	}

	fn select(&mut self, exec: Exec) {
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

		#[allow(clippy::single_match)]
		match exec.cmd.as_str() {
			"help" => self.app.cx.help.toggle(Layer::Select),
			_ => {}
		}
	}

	fn input(&mut self, exec: Exec) {
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

				#[allow(clippy::single_match)]
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

	fn help(&mut self, exec: Exec) {
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

		#[allow(clippy::single_match)]
		match exec.cmd.as_str() {
			"close" => self.app.cx.help.toggle(Layer::Help),
			_ => {}
		}
	}

	fn completion(&mut self, exec: Exec) {
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

		#[allow(clippy::single_match)]
		match exec.cmd.as_str() {
			"help" => self.app.cx.help.toggle(Layer::Completion),
			"close_input" => self.app.cx.input.close(exec),
			_ => {}
		}
	}
}
