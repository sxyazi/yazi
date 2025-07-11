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
					return self.app.core.mgr.$name(cmd, $($args),*);
				}
			};
			(ACTIVE, $name:ident $(,$args:expr)*) => {
				if cmd.name == stringify!($name) {
					return if let Some(tab) = cmd.get("tab") {
						let Some(id) = tab.as_id() else { return };
						let Some(tab) = self.app.core.mgr.tabs.find_mut(id) else { return };
						tab.$name(cmd, $($args),*)
					} else {
						self.app.core.mgr.active_mut().$name(cmd, $($args),*)
					};
				}
			};
			(TABS, $name:ident) => {
				if cmd.name == concat!("tab_", stringify!($name)) {
					return self.app.core.mgr.tabs.$name(cmd);
				}
			};
		}

		on!(MGR, update_tasks);
		on!(MGR, update_files, &self.app.core.tasks);
		on!(MGR, update_mimes, &self.app.core.tasks);
		on!(MGR, update_paged, &self.app.core.tasks);
		on!(MGR, update_yanked);
		on!(MGR, watch);
		on!(MGR, peek);
		on!(MGR, seek);
		on!(MGR, spot);
		on!(MGR, refresh, &self.app.core.tasks);
		on!(MGR, quit, &self.app.core.tasks);
		on!(MGR, close, &self.app.core.tasks);
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
		on!(MGR, open, &self.app.core.tasks);
		on!(MGR, open_do, &self.app.core.tasks);
		on!(MGR, yank);
		on!(MGR, unyank);
		on!(MGR, paste, &self.app.core.tasks);
		on!(MGR, link, &self.app.core.tasks);
		on!(MGR, hardlink, &self.app.core.tasks);
		on!(MGR, remove, &self.app.core.tasks);
		on!(MGR, remove_do, &self.app.core.tasks);
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
		on!(ACTIVE, sort, &self.app.core.tasks);

		// Tabs
		on!(TABS, create); // TODO: use `tab_create` instead
		on!(TABS, close);
		on!(TABS, switch);
		on!(TABS, swap);

		match cmd.name.as_ref() {
			// Help
			"help" => self.app.core.help.toggle(Layer::Mgr),
			// Plugin
			"plugin" => self.app.plugin(cmd),
			_ => {}
		}
	}

	fn tasks(&mut self, cmd: CmdCow) {
		macro_rules! on {
			($name:ident) => {
				if cmd.name == stringify!($name) {
					return self.app.core.tasks.$name(cmd);
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

		match cmd.name.as_ref() {
			// Help
			"help" => self.app.core.help.toggle(Layer::Tasks),
			// Plugin
			"plugin" => self.app.plugin(cmd),
			_ => {}
		}
	}

	fn spot(&mut self, cmd: CmdCow) {
		macro_rules! on {
			($name:ident) => {
				if cmd.name == stringify!($name) {
					return self.app.core.active_mut().spot.$name(cmd);
				}
			};
		}

		on!(arrow);
		on!(close);
		on!(swipe);
		on!(copy);

		match cmd.name.as_ref() {
			// Help
			"help" => self.app.core.help.toggle(Layer::Spot),
			// Plugin
			"plugin" => self.app.plugin(cmd),
			_ => {}
		}
	}

	fn pick(&mut self, cmd: CmdCow) {
		macro_rules! on {
			($name:ident) => {
				if cmd.name == stringify!($name) {
					return self.app.core.pick.$name(cmd);
				}
			};
		}

		on!(show);
		on!(close);
		on!(arrow);

		match cmd.name.as_ref() {
			// Help
			"help" => self.app.core.help.toggle(Layer::Pick),
			// Plugin
			"plugin" => self.app.plugin(cmd),
			_ => {}
		}
	}

	fn input(&mut self, cmd: CmdCow) {
		macro_rules! on {
			($name:ident) => {
				if cmd.name == stringify!($name) {
					return self.app.core.input.$name(cmd);
				}
			};
		}

		on!(escape);
		on!(show);
		on!(close);

		match self.app.core.input.mode() {
			InputMode::Normal => {
				match cmd.name.as_ref() {
					// Help
					"help" => return self.app.core.help.toggle(Layer::Input),
					// Plugin
					"plugin" => return self.app.plugin(cmd),
					_ => {}
				}
			}
			InputMode::Insert | InputMode::Replace => {}
		};

		self.app.core.input.execute(cmd)
	}

	fn confirm(&mut self, cmd: CmdCow) {
		macro_rules! on {
			($name:ident $(,$args:expr)*) => {
				if cmd.name == stringify!($name) {
					return self.app.core.confirm.$name(cmd, $($args),*);
				}
			};
		}

		on!(arrow, &self.app.core.mgr);
		on!(show);
		on!(close);
	}

	fn help(&mut self, cmd: CmdCow) {
		macro_rules! on {
			($name:ident) => {
				if cmd.name == stringify!($name) {
					return self.app.core.help.$name(cmd);
				}
			};
		}

		on!(escape);
		on!(arrow);
		on!(filter);

		match cmd.name.as_ref() {
			"close" => self.app.core.help.toggle(Layer::Help),
			// Plugin
			"plugin" => self.app.plugin(cmd),
			_ => {}
		}
	}

	fn cmp(&mut self, cmd: CmdCow) {
		macro_rules! on {
			($name:ident) => {
				if cmd.name == stringify!($name) {
					return self.app.core.cmp.$name(cmd);
				}
			};
		}

		on!(trigger);
		on!(show);
		on!(close);
		on!(arrow);

		match cmd.name.as_ref() {
			// Help
			"help" => self.app.core.help.toggle(Layer::Cmp),
			// Plugin
			"plugin" => self.app.plugin(cmd),
			_ => {}
		}
	}

	fn which(&mut self, cmd: CmdCow) {
		macro_rules! on {
			($name:ident) => {
				if cmd.name == stringify!($name) {
					return self.app.core.which.$name(cmd);
				}
			};
		}

		on!(show);
		on!(callback);
	}
}
