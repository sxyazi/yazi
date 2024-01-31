use yazi_core::input::InputMode;
use yazi_shared::{event::Cmd, Layer};

use crate::app::App;

pub(super) struct Executor<'a> {
	app: &'a mut App,
}

impl<'a> Executor<'a> {
	#[inline]
	pub(super) fn new(app: &'a mut App) -> Self { Self { app } }

	#[inline]
	pub(super) fn execute(&mut self, cmd: Cmd, layer: Layer) {
		match layer {
			Layer::App => self.app(cmd),
			Layer::Manager => self.manager(cmd),
			Layer::Tasks => self.tasks(cmd),
			Layer::Select => self.select(cmd),
			Layer::Input => self.input(cmd),
			Layer::Help => self.help(cmd),
			Layer::Completion => self.completion(cmd),
			Layer::Which => unreachable!(),
		}
	}

	fn app(&mut self, cmd: Cmd) {
		macro_rules! on {
			($name:ident) => {
				if cmd.name == stringify!($name) {
					return self.app.$name(cmd);
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

	fn manager(&mut self, cmd: Cmd) {
		macro_rules! on {
			(MANAGER, $name:ident $(,$args:expr)*) => {
				if cmd.name == stringify!($name) {
					return self.app.cx.manager.$name(cmd, $($args),*);
				}
			};
			(ACTIVE, $name:ident $(,$args:expr)*) => {
				if cmd.name == stringify!($name) {
					return self.app.cx.manager.active_mut().$name(cmd, $($args),*);
				}
			};
			(TABS, $name:ident) => {
				if cmd.name == concat!("tab_", stringify!($name)) {
					return self.app.cx.manager.tabs.$name(cmd);
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

		match cmd.name.as_bytes() {
			// Tasks
			b"tasks_show" => self.app.cx.tasks.toggle(()),
			// Help
			b"help" => self.app.cx.help.toggle(Layer::Manager),
			// Plugin
			b"plugin" => self.app.plugin(cmd),
			_ => {}
		}
	}

	fn tasks(&mut self, cmd: Cmd) {
		macro_rules! on {
			($name:ident) => {
				if cmd.name == stringify!($name) {
					return self.app.cx.tasks.$name(cmd);
				}
			};
			($name:ident, $alias:literal) => {
				if cmd.name == $alias {
					return self.app.cx.tasks.$name(cmd);
				}
			};
		}

		on!(open);
		on!(toggle, "close");
		on!(arrow);
		on!(inspect);
		on!(cancel);

		#[allow(clippy::single_match)]
		match cmd.name.as_str() {
			"help" => self.app.cx.help.toggle(Layer::Tasks),
			_ => {}
		}
	}

	fn select(&mut self, cmd: Cmd) {
		macro_rules! on {
			($name:ident) => {
				if cmd.name == stringify!($name) {
					return self.app.cx.select.$name(cmd);
				}
			};
		}

		on!(show);
		on!(close);
		on!(arrow);

		#[allow(clippy::single_match)]
		match cmd.name.as_str() {
			"help" => self.app.cx.help.toggle(Layer::Select),
			_ => {}
		}
	}

	fn input(&mut self, cmd: Cmd) {
		macro_rules! on {
			($name:ident) => {
				if cmd.name == stringify!($name) {
					return self.app.cx.input.$name(cmd);
				}
			};
			($name:ident, $alias:literal) => {
				if cmd.name == $alias {
					return self.app.cx.input.$name(cmd);
				}
			};
		}

		on!(show);
		on!(close);
		on!(escape);
		on!(move_, "move");
		on!(backward);
		on!(forward);

		if cmd.name.as_str() == "complete" {
			return if cmd.named.contains_key("trigger") {
				self.app.cx.completion.trigger(cmd)
			} else {
				self.app.cx.input.complete(cmd)
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
				match cmd.name.as_str() {
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

	fn help(&mut self, cmd: Cmd) {
		macro_rules! on {
			($name:ident) => {
				if cmd.name == stringify!($name) {
					return self.app.cx.help.$name(cmd);
				}
			};
		}

		on!(escape);
		on!(arrow);
		on!(filter);

		#[allow(clippy::single_match)]
		match cmd.name.as_str() {
			"close" => self.app.cx.help.toggle(Layer::Help),
			_ => {}
		}
	}

	fn completion(&mut self, cmd: Cmd) {
		macro_rules! on {
			($name:ident) => {
				if cmd.name == stringify!($name) {
					return self.app.cx.completion.$name(cmd);
				}
			};
		}

		on!(trigger);
		on!(show);
		on!(close);
		on!(arrow);

		#[allow(clippy::single_match)]
		match cmd.name.as_str() {
			"help" => self.app.cx.help.toggle(Layer::Completion),
			"close_input" => self.app.cx.input.close(cmd),
			_ => {}
		}
	}
}
