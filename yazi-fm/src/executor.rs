use yazi_core::input::InputMode;
use yazi_shared::{Layer, event::CmdCow};

use crate::app::App;

pub(super) struct Executor<'a> {
	app: &'a mut App,
}

impl<'a> Executor<'a> {
	#[inline]
	pub(super) fn new(app: &'a mut App) -> Self { Self { app } }

	#[inline]
	pub(super) fn execute(&mut self, cmd: CmdCow, layer: Layer) {
		match layer {
			Layer::App => self.app(cmd),
			Layer::Manager => self.manager(cmd),
			Layer::Tasks => self.tasks(cmd),
			Layer::Spot => self.spot(cmd),
			Layer::Pick => self.pick(cmd),
			Layer::Input => self.input(cmd),
			Layer::Confirm => self.confirm(cmd),
			Layer::Help => self.help(cmd),
			Layer::Completion => self.completion(cmd),
			Layer::Which => self.which(cmd),
		}
	}

	fn app(&mut self, cmd: CmdCow) {
		macro_rules! on {
			($name:ident) => {
				if cmd.name == stringify!($name) {
					return self.app.$name(cmd);
				}
			};
		}

		on!(accept_payload);
		on!(notify);
		on!(plugin);
		on!(plugin_do);
		on!(update_notify);
		on!(update_progress);
		on!(resize);
		on!(stop);
		on!(resume);
	}

	fn manager(&mut self, cmd: CmdCow) {
		macro_rules! on {
			(MANAGER, $name:ident $(,$args:expr)*) => {
				if cmd.name == stringify!($name) {
					return self.app.cx.manager.$name(cmd, $($args),*);
				}
			};
			(ACTIVE, $name:ident $(,$args:expr)*) => {
				if cmd.name == stringify!($name) {
					return if let Some(tab) = cmd.get("tab") {
						let Some(id) = tab.as_id() else { return };
						let Some(tab) = self.app.cx.manager.tabs.find_mut(id) else { return };
						tab.$name(cmd, $($args),*)
					} else {
						self.app.cx.manager.active_mut().$name(cmd, $($args),*)
					};
				}
			};
			(TABS, $name:ident) => {
				if cmd.name == concat!("tab_", stringify!($name)) {
					return self.app.cx.manager.tabs.$name(cmd);
				}
			};
		}

		on!(MANAGER, update_tasks);
		on!(MANAGER, update_files, &self.app.cx.tasks);
		on!(MANAGER, update_mimes, &self.app.cx.tasks);
		on!(MANAGER, update_paged, &self.app.cx.tasks);
		on!(MANAGER, update_yanked);
		on!(MANAGER, hover);
		on!(MANAGER, peek);
		on!(MANAGER, seek);
		on!(MANAGER, spot);
		on!(MANAGER, refresh, &self.app.cx.tasks);
		on!(MANAGER, quit, &self.app.cx.tasks);
		on!(MANAGER, close, &self.app.cx.tasks);
		on!(MANAGER, suspend);
		on!(ACTIVE, escape);
		on!(ACTIVE, update_peeked);
		on!(ACTIVE, update_spotted);

		// Navigation
		on!(ACTIVE, arrow);
		on!(ACTIVE, leave);
		on!(ACTIVE, enter);
		on!(ACTIVE, back);
		on!(ACTIVE, forward);
		on!(ACTIVE, cd);
		on!(ACTIVE, reveal);

		// Toggle
		on!(ACTIVE, toggle);
		on!(ACTIVE, toggle_all);
		on!(ACTIVE, visual_mode);

		// Operation
		on!(MANAGER, open, &self.app.cx.tasks);
		on!(MANAGER, open_do, &self.app.cx.tasks);
		on!(MANAGER, yank);
		on!(MANAGER, unyank);
		on!(MANAGER, paste, &self.app.cx.tasks);
		on!(MANAGER, link, &self.app.cx.tasks);
		on!(MANAGER, hardlink, &self.app.cx.tasks);
		on!(MANAGER, remove, &self.app.cx.tasks);
		on!(MANAGER, remove_do, &self.app.cx.tasks);
		on!(MANAGER, create);
		on!(MANAGER, bulk_create);

		on!(MANAGER, rename);
		on!(ACTIVE, copy);
		on!(ACTIVE, shell);
		on!(ACTIVE, hidden);
		on!(ACTIVE, linemode);
		on!(ACTIVE, search);
		on!(ACTIVE, search_do);

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

		match cmd.name.as_str() {
			// Tasks
			"tasks_show" => self.app.cx.tasks.toggle(()),
			// Help
			"help" => self.app.cx.help.toggle(Layer::Manager),
			// Plugin
			"plugin" => self.app.plugin(cmd),
			_ => {}
		}
	}

	fn tasks(&mut self, cmd: CmdCow) {
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

		on!(toggle, "close");
		on!(arrow);
		on!(inspect);
		on!(cancel);
		on!(open_with);
		on!(process_exec);

		match cmd.name.as_str() {
			// Help
			"help" => self.app.cx.help.toggle(Layer::Tasks),
			// Plugin
			"plugin" => self.app.plugin(cmd),
			_ => {}
		}
	}

	fn spot(&mut self, cmd: CmdCow) {
		macro_rules! on {
			($name:ident) => {
				if cmd.name == stringify!($name) {
					return self.app.cx.active_mut().spot.$name(cmd);
				}
			};
		}

		on!(arrow);
		on!(close);
		on!(swipe);
		on!(copy);

		match cmd.name.as_str() {
			// Help
			"help" => self.app.cx.help.toggle(Layer::Spot),
			// Plugin
			"plugin" => self.app.plugin(cmd),
			_ => {}
		}
	}

	fn pick(&mut self, cmd: CmdCow) {
		macro_rules! on {
			($name:ident) => {
				if cmd.name == stringify!($name) {
					return self.app.cx.pick.$name(cmd);
				}
			};
		}

		on!(show);
		on!(close);
		on!(arrow);

		match cmd.name.as_str() {
			// Help
			"help" => self.app.cx.help.toggle(Layer::Pick),
			// Plugin
			"plugin" => self.app.plugin(cmd),
			_ => {}
		}
	}

	fn input(&mut self, cmd: CmdCow) {
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
			return if cmd.bool("trigger") {
				self.app.cx.completion.trigger(cmd)
			} else {
				self.app.cx.input.complete(cmd)
			};
		}

		match self.app.cx.input.mode() {
			InputMode::Normal => {
				on!(insert);
				on!(visual);
				on!(replace);

				on!(delete);
				on!(yank);
				on!(paste);

				on!(undo);
				on!(redo);

				match cmd.name.as_str() {
					// Help
					"help" => self.app.cx.help.toggle(Layer::Input),
					// Plugin
					"plugin" => self.app.plugin(cmd),
					_ => {}
				}
			}
			InputMode::Insert => {
				on!(backspace);
				on!(kill);
			}
			InputMode::Replace => {}
		}
	}

	fn confirm(&mut self, cmd: CmdCow) {
		macro_rules! on {
			($name:ident $(,$args:expr)*) => {
				if cmd.name == stringify!($name) {
					return self.app.cx.confirm.$name(cmd, $($args),*);
				}
			};
		}

		on!(arrow, &self.app.cx.manager);
		on!(show);
		on!(close);
	}

	fn help(&mut self, cmd: CmdCow) {
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

		match cmd.name.as_str() {
			"close" => self.app.cx.help.toggle(Layer::Help),
			// Plugin
			"plugin" => self.app.plugin(cmd),
			_ => {}
		}
	}

	fn completion(&mut self, cmd: CmdCow) {
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

		match cmd.name.as_str() {
			"close_input" => self.app.cx.input.close(cmd),
			// Help
			"help" => self.app.cx.help.toggle(Layer::Completion),
			// Plugin
			"plugin" => self.app.plugin(cmd),
			_ => {}
		}
	}

	fn which(&mut self, cmd: CmdCow) {
		macro_rules! on {
			($name:ident) => {
				if cmd.name == stringify!($name) {
					return self.app.cx.which.$name(cmd);
				}
			};
		}

		on!(show);
		on!(callback);
	}
}
