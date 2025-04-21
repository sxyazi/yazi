use yazi_shared::{Layer, event::CmdCow};
use yazi_widgets::input::InputMode;

use crate::app::App;

pub(super) struct Executor<'a> {
	app: &'a mut App,
}

impl<'a> Executor<'a> {
	#[inline]
	pub(super) fn new(app: &'a mut App) -> Self { Self { app } }

	#[inline]
	pub(super) fn execute(&mut self, cmd: CmdCow) {
		match cmd.layer {
			Layer::App => self.app(cmd),
			Layer::Mgr => self.mgr(cmd),
			Layer::Tasks => self.tasks(cmd),
			Layer::Spot => self.spot(cmd),
			Layer::Pick => self.pick(cmd),
			Layer::Input => self.input(cmd),
			Layer::Confirm => self.confirm(cmd),
			Layer::Help => self.help(cmd),
			Layer::Cmp => self.cmp(cmd),
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

	fn mgr(&mut self, cmd: CmdCow) {
		macro_rules! on {
			(MGR, $name:ident $(,$args:expr)*) => {
				if cmd.name == stringify!($name) {
					return self.app.cx.mgr.$name(cmd, $($args),*);
				}
			};
			(ACTIVE, $name:ident $(,$args:expr)*) => {
				if cmd.name == stringify!($name) {
					return if let Some(tab) = cmd.get("tab") {
						let Some(id) = tab.as_id() else { return };
						let Some(tab) = self.app.cx.mgr.tabs.find_mut(id) else { return };
						tab.$name(cmd, $($args),*)
					} else {
						self.app.cx.mgr.active_mut().$name(cmd, $($args),*)
					};
				}
			};
			(TABS, $name:ident) => {
				if cmd.name == concat!("tab_", stringify!($name)) {
					return self.app.cx.mgr.tabs.$name(cmd);
				}
			};
		}

		on!(MGR, update_tasks);
		on!(MGR, update_files, &self.app.cx.tasks);
		on!(MGR, update_mimes, &self.app.cx.tasks);
		on!(MGR, update_paged, &self.app.cx.tasks);
		on!(MGR, update_yanked);
		on!(MGR, watch);
		on!(MGR, peek);
		on!(MGR, seek);
		on!(MGR, spot);
		on!(MGR, refresh, &self.app.cx.tasks);
		on!(MGR, quit, &self.app.cx.tasks);
		on!(MGR, close, &self.app.cx.tasks);
		on!(MGR, suspend);
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
		on!(ACTIVE, follow);

		// Toggle
		on!(ACTIVE, toggle);
		on!(ACTIVE, toggle_all);
		on!(ACTIVE, visual_mode);

		// Operation
		on!(MGR, open, &self.app.cx.tasks);
		on!(MGR, open_do, &self.app.cx.tasks);
		on!(MGR, yank);
		on!(MGR, unyank);
		on!(MGR, paste, &self.app.cx.tasks);
		on!(MGR, link, &self.app.cx.tasks);
		on!(MGR, hardlink, &self.app.cx.tasks);
		on!(MGR, remove, &self.app.cx.tasks);
		on!(MGR, remove_do, &self.app.cx.tasks);
		on!(MGR, create);
		on!(MGR, rename);
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
			// Help
			"help" => self.app.cx.help.toggle(Layer::Mgr),
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

		on!(show);
		on!(close);
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
		}

		on!(escape);
		on!(show);
		on!(close);

		match self.app.cx.input.mode() {
			InputMode::Normal => {
				match cmd.name.as_str() {
					// Help
					"help" => return self.app.cx.help.toggle(Layer::Input),
					// Plugin
					"plugin" => return self.app.plugin(cmd),
					_ => {}
				}
			}
			InputMode::Insert => match cmd.name.as_str() {
				"complete" if cmd.bool("trigger") => return self.app.cx.cmp.trigger(cmd),
				_ => {}
			},
			InputMode::Replace => {}
		};

		self.app.cx.input.execute(cmd)
	}

	fn confirm(&mut self, cmd: CmdCow) {
		macro_rules! on {
			($name:ident $(,$args:expr)*) => {
				if cmd.name == stringify!($name) {
					return self.app.cx.confirm.$name(cmd, $($args),*);
				}
			};
		}

		on!(arrow, &self.app.cx.mgr);
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

	fn cmp(&mut self, cmd: CmdCow) {
		macro_rules! on {
			($name:ident) => {
				if cmd.name == stringify!($name) {
					return self.app.cx.cmp.$name(cmd);
				}
			};
		}

		on!(trigger);
		on!(show);
		on!(close);
		on!(arrow);

		match cmd.name.as_str() {
			// Help
			"help" => self.app.cx.help.toggle(Layer::Cmp),
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
