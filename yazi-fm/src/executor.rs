use anyhow::Result;
use yazi_actor::Ctx;
use yazi_macro::{act, succ};
use yazi_shared::{Layer, data::Data, event::CmdCow};
use yazi_widgets::input::InputMode;

use crate::app::App;

pub(super) struct Executor<'a> {
	app: &'a mut App,
}

impl<'a> Executor<'a> {
	#[inline]
	pub(super) fn new(app: &'a mut App) -> Self { Self { app } }

	#[inline]
	pub(super) fn execute(&mut self, cmd: CmdCow) -> Result<Data> {
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

	fn app(&mut self, cmd: CmdCow) -> Result<Data> {
		macro_rules! on {
			($name:ident) => {
				if cmd.name == stringify!($name) {
					return act!($name, self.app, cmd);
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
		on!(deprecate);

		succ!();
	}

	fn mgr(&mut self, cmd: CmdCow) -> Result<Data> {
		let cx = &mut Ctx::new(&mut self.app.core, &cmd)?;

		macro_rules! on {
			($name:ident) => {
				if cmd.name == stringify!($name) {
					return act!(mgr:$name, cx, cmd)
				}
			};
		}

		on!(cd);
		on!(update_yanked);

		on!(update_files);
		on!(update_mimes);
		on!(update_paged);
		on!(watch);
		on!(peek);
		on!(seek);
		on!(spot);
		on!(refresh);
		on!(quit);
		on!(close);
		on!(suspend);
		on!(escape);
		on!(update_peeked);
		on!(update_spotted);

		// Navigation
		on!(arrow);
		on!(leave);
		on!(enter);
		on!(back);
		on!(forward);
		on!(reveal);
		on!(follow);

		// Toggle
		on!(toggle);
		on!(toggle_all);
		on!(visual_mode);

		// Operation
		on!(open);
		on!(open_do);
		on!(yank);
		on!(unyank);
		on!(paste);
		on!(link);
		on!(hardlink);
		on!(remove);
		on!(remove_do);
		on!(create);
		on!(rename);
		on!(copy);
		on!(shell);
		on!(hidden);
		on!(linemode);
		on!(search);
		on!(search_do);
		on!(bulk_rename);

		// Filter
		on!(filter);
		on!(filter_do);

		// Find
		on!(find);
		on!(find_do);
		on!(find_arrow);

		// Sorting
		on!(sort);

		// Tabs
		on!(tab_create);
		on!(tab_close);
		on!(tab_switch);
		on!(tab_swap);

		// VFS
		on!(download);
		on!(upload);
		on!(displace_do);

		match cmd.name.as_ref() {
			// Help
			"help" => act!(help:toggle, cx, Layer::Mgr),
			// Plugin
			"plugin" => act!(plugin, self.app, cmd),
			_ => succ!(),
		}
	}

	fn tasks(&mut self, cmd: CmdCow) -> Result<Data> {
		let cx = &mut Ctx::new(&mut self.app.core, &cmd)?;

		macro_rules! on {
			($name:ident) => {
				if cmd.name == stringify!($name) {
					return act!(tasks:$name, cx, cmd);
				}
			};
		}

		on!(update_succeed);

		on!(show);
		on!(close);
		on!(arrow);
		on!(inspect);
		on!(cancel);
		on!(process_open);
		on!(open_shell_compat);

		match cmd.name.as_ref() {
			// Help
			"help" => act!(help:toggle, cx, Layer::Tasks),
			// Plugin
			"plugin" => act!(plugin, self.app, cmd),
			_ => succ!(),
		}
	}

	fn spot(&mut self, cmd: CmdCow) -> Result<Data> {
		let cx = &mut Ctx::new(&mut self.app.core, &cmd)?;

		macro_rules! on {
			($name:ident) => {
				if cmd.name == stringify!($name) {
					return act!(spot:$name, cx, cmd);
				}
			};
		}

		on!(arrow);
		on!(close);
		on!(swipe);
		on!(copy);

		match cmd.name.as_ref() {
			// Help
			"help" => act!(help:toggle, cx, Layer::Spot),
			// Plugin
			"plugin" => act!(plugin, self.app, cmd),
			_ => succ!(),
		}
	}

	fn pick(&mut self, cmd: CmdCow) -> Result<Data> {
		let cx = &mut Ctx::new(&mut self.app.core, &cmd)?;

		macro_rules! on {
			($name:ident) => {
				if cmd.name == stringify!($name) {
					return act!(pick:$name, cx, cmd);
				}
			};
		}

		on!(show);
		on!(close);
		on!(arrow);

		match cmd.name.as_ref() {
			// Help
			"help" => act!(help:toggle, cx, Layer::Pick),
			// Plugin
			"plugin" => act!(plugin, self.app, cmd),
			_ => succ!(),
		}
	}

	fn input(&mut self, cmd: CmdCow) -> Result<Data> {
		let mode = self.app.core.input.mode();
		let cx = &mut Ctx::new(&mut self.app.core, &cmd)?;

		macro_rules! on {
			($name:ident) => {
				if cmd.name == stringify!($name) {
					return act!(input:$name, cx, cmd);
				}
			};
		}

		on!(escape);
		on!(show);
		on!(close);

		match mode {
			InputMode::Normal => {
				match cmd.name.as_ref() {
					// Help
					"help" => return act!(help:toggle, cx, Layer::Input),
					// Plugin
					"plugin" => return act!(plugin, self.app, cmd),
					_ => {}
				}
			}
			InputMode::Insert => {
				on!(complete);
			}
			InputMode::Replace => {}
		};

		self.app.core.input.execute(cmd)
	}

	fn confirm(&mut self, cmd: CmdCow) -> Result<Data> {
		let cx = &mut Ctx::new(&mut self.app.core, &cmd)?;

		macro_rules! on {
			($name:ident) => {
				if cmd.name == stringify!($name) {
					return act!(confirm:$name, cx, cmd);
				}
			};
		}

		on!(arrow);
		on!(show);
		on!(close);

		succ!();
	}

	fn help(&mut self, cmd: CmdCow) -> Result<Data> {
		let cx = &mut Ctx::new(&mut self.app.core, &cmd)?;

		macro_rules! on {
			($name:ident) => {
				if cmd.name == stringify!($name) {
					return act!(help:$name, cx, cmd);
				}
			};
		}

		on!(escape);
		on!(arrow);
		on!(filter);

		match cmd.name.as_ref() {
			// Help
			"close" => act!(help:toggle, cx, Layer::Help),
			// Plugin
			"plugin" => act!(plugin, self.app, cmd),
			_ => succ!(),
		}
	}

	fn cmp(&mut self, cmd: CmdCow) -> Result<Data> {
		let cx = &mut Ctx::new(&mut self.app.core, &cmd)?;

		macro_rules! on {
			($name:ident) => {
				if cmd.name == stringify!($name) {
					return act!(cmp:$name, cx, cmd);
				}
			};
		}

		on!(trigger);
		on!(show);
		on!(close);
		on!(arrow);

		match cmd.name.as_ref() {
			// Help
			"help" => act!(help:toggle, cx, Layer::Cmp),
			// Plugin
			"plugin" => act!(plugin, self.app, cmd),
			_ => succ!(),
		}
	}

	fn which(&mut self, cmd: CmdCow) -> Result<Data> {
		let cx = &mut Ctx::new(&mut self.app.core, &cmd)?;

		macro_rules! on {
			($name:ident) => {
				if cmd.name == stringify!($name) {
					return act!(which:$name, cx, cmd);
				}
			};
		}

		on!(show);
		on!(callback);

		succ!();
	}
}
