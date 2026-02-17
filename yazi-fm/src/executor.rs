use anyhow::{Context, Result};
use yazi_actor::Ctx;
use yazi_macro::{act, succ};
use yazi_shared::{Layer, data::Data, event::ActionCow};
use yazi_widgets::input::InputMode;

use crate::app::App;

pub(super) struct Executor<'a> {
	app: &'a mut App,
}

impl<'a> Executor<'a> {
	#[inline]
	pub(super) fn new(app: &'a mut App) -> Self { Self { app } }

	#[inline]
	pub(super) fn execute(&mut self, action: ActionCow) -> Result<Data> {
		match action.layer {
			Layer::App => self.app(action),
			Layer::Mgr => self.mgr(action),
			Layer::Tasks => self.tasks(action),
			Layer::Spot => self.spot(action),
			Layer::Pick => self.pick(action),
			Layer::Input => self.input(action),
			Layer::Confirm => self.confirm(action),
			Layer::Help => self.help(action),
			Layer::Cmp => self.cmp(action),
			Layer::Which => self.which(action),
			Layer::Notify => self.notify(action),
		}
	}

	fn app(&mut self, mut action: ActionCow) -> Result<Data> {
		let cx = &mut Ctx::new(&action, &mut self.app.core, &mut self.app.term)?;

		macro_rules! on {
			($name:ident) => {
				if action.name == stringify!($name) {
					return act!(app:$name, cx, action);
				}
			};
		}

		on!(accept_payload);
		on!(plugin);
		on!(plugin_do);
		on!(update_progress);
		on!(deprecate);
		on!(quit);

		match &*action.name {
			"resize" => act!(app:resize, cx, crate::Root::reflow as fn(_) -> _),
			"resume" => act!(app:resume, cx, yazi_parser::app::ResumeOpt {
				tx: self.app.signals.tx.clone(),
				token: action.take_any("token").context("Invalid 'token' in ResumeOpt")?,
				reflow: crate::Root::reflow,
			}),
			"stop" => act!(app:stop, cx, yazi_parser::app::StopOpt {
				tx: self.app.signals.tx.clone(),
				token: action.take_any("token").context("Invalid 'token' in StopOpt")?,
			}),
			_ => succ!(),
		}
	}

	fn mgr(&mut self, action: ActionCow) -> Result<Data> {
		let cx = &mut Ctx::new(&action, &mut self.app.core, &mut self.app.term)?;

		macro_rules! on {
			($name:ident) => {
				if action.name == stringify!($name) {
					return act!(mgr:$name, cx, action)
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
		on!(tab_rename);
		on!(tab_close);
		on!(tab_switch);
		on!(tab_swap);

		// VFS
		on!(download);
		on!(upload);
		on!(displace_do);

		match action.name.as_ref() {
			// Help
			"help" => act!(help:toggle, cx, Layer::Mgr),
			// Plugin
			"plugin" => act!(app:plugin, cx, action),
			_ => succ!(),
		}
	}

	fn tasks(&mut self, action: ActionCow) -> Result<Data> {
		let cx = &mut Ctx::new(&action, &mut self.app.core, &mut self.app.term)?;

		macro_rules! on {
			($name:ident) => {
				if action.name == stringify!($name) {
					return act!(tasks:$name, cx, action);
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

		match action.name.as_ref() {
			// Help
			"help" => act!(help:toggle, cx, Layer::Tasks),
			// Plugin
			"plugin" => act!(app:plugin, cx, action),
			_ => succ!(),
		}
	}

	fn spot(&mut self, action: ActionCow) -> Result<Data> {
		let cx = &mut Ctx::new(&action, &mut self.app.core, &mut self.app.term)?;

		macro_rules! on {
			($name:ident) => {
				if action.name == stringify!($name) {
					return act!(spot:$name, cx, action);
				}
			};
		}

		on!(arrow);
		on!(close);
		on!(swipe);
		on!(copy);

		match action.name.as_ref() {
			// Help
			"help" => act!(help:toggle, cx, Layer::Spot),
			// Plugin
			"plugin" => act!(app:plugin, cx, action),
			_ => succ!(),
		}
	}

	fn pick(&mut self, action: ActionCow) -> Result<Data> {
		let cx = &mut Ctx::new(&action, &mut self.app.core, &mut self.app.term)?;

		macro_rules! on {
			($name:ident) => {
				if action.name == stringify!($name) {
					return act!(pick:$name, cx, action);
				}
			};
		}

		on!(show);
		on!(close);
		on!(arrow);

		match action.name.as_ref() {
			// Help
			"help" => act!(help:toggle, cx, Layer::Pick),
			// Plugin
			"plugin" => act!(app:plugin, cx, action),
			_ => succ!(),
		}
	}

	fn input(&mut self, action: ActionCow) -> Result<Data> {
		let mode = self.app.core.input.mode();
		let cx = &mut Ctx::new(&action, &mut self.app.core, &mut self.app.term)?;

		macro_rules! on {
			($name:ident) => {
				if action.name == stringify!($name) {
					return act!(input:$name, cx, action);
				}
			};
		}

		on!(escape);
		on!(show);
		on!(close);

		match mode {
			InputMode::Normal => {
				match action.name.as_ref() {
					// Help
					"help" => return act!(help:toggle, cx, Layer::Input),
					// Plugin
					"plugin" => return act!(app:plugin, cx, action),
					_ => {}
				}
			}
			InputMode::Insert => {
				on!(complete);
			}
			InputMode::Replace => {}
		};

		self.app.core.input.execute(action)
	}

	fn confirm(&mut self, action: ActionCow) -> Result<Data> {
		let cx = &mut Ctx::new(&action, &mut self.app.core, &mut self.app.term)?;

		macro_rules! on {
			($name:ident) => {
				if action.name == stringify!($name) {
					return act!(confirm:$name, cx, action);
				}
			};
		}

		on!(arrow);
		on!(show);
		on!(close);

		succ!();
	}

	fn help(&mut self, action: ActionCow) -> Result<Data> {
		let cx = &mut Ctx::new(&action, &mut self.app.core, &mut self.app.term)?;

		macro_rules! on {
			($name:ident) => {
				if action.name == stringify!($name) {
					return act!(help:$name, cx, action);
				}
			};
		}

		on!(escape);
		on!(arrow);
		on!(filter);

		match action.name.as_ref() {
			// Help
			"close" => act!(help:toggle, cx, Layer::Help),
			// Plugin
			"plugin" => act!(app:plugin, cx, action),
			_ => succ!(),
		}
	}

	fn cmp(&mut self, action: ActionCow) -> Result<Data> {
		let cx = &mut Ctx::new(&action, &mut self.app.core, &mut self.app.term)?;

		macro_rules! on {
			($name:ident) => {
				if action.name == stringify!($name) {
					return act!(cmp:$name, cx, action);
				}
			};
		}

		on!(trigger);
		on!(show);
		on!(close);
		on!(arrow);

		match action.name.as_ref() {
			// Help
			"help" => act!(help:toggle, cx, Layer::Cmp),
			// Plugin
			"plugin" => act!(app:plugin, cx, action),
			_ => succ!(),
		}
	}

	fn which(&mut self, action: ActionCow) -> Result<Data> {
		let cx = &mut Ctx::new(&action, &mut self.app.core, &mut self.app.term)?;

		macro_rules! on {
			($name:ident) => {
				if action.name == stringify!($name) {
					return act!(which:$name, cx, action);
				}
			};
		}

		on!(activate);
		on!(dismiss);

		succ!();
	}

	fn notify(&mut self, action: ActionCow) -> Result<Data> {
		let cx = &mut Ctx::new(&action, &mut self.app.core, &mut self.app.term)?;

		macro_rules! on {
			($name:ident) => {
				if action.name == stringify!($name) {
					return act!(notify:$name, cx, action);
				}
			};
		}

		on!(push);
		on!(tick);

		succ!();
	}
}
