use std::ffi::OsString;

use anyhow::Result;
use yazi_config::ARGS;
use yazi_shared::{event::QuitAction, term::Term};

use crate::app::App;

impl App {
	pub(crate) fn quit(&mut self, quit_actions: Vec<QuitAction>) -> Result<()> {
		if quit_actions.contains(&QuitAction::None) {
			Term::goodbye(|| false)
		}

		for quit_action in quit_actions {
			match quit_action {
				QuitAction::None => unreachable!(),
				QuitAction::CwdToFile => self.cwd_to_file(),
				QuitAction::SelectToFile(selected) => self.select_to_file(selected),
			}
		}

		Term::goodbye(|| false);
	}

	fn cwd_to_file(&self) {
		if let Some(p) = ARGS.cwd_file.as_ref() {
			let cwd = self.cx.manager.cwd().as_os_str();
			std::fs::write(p, cwd.as_encoded_bytes()).ok();
		}
	}

	fn select_to_file(&self, selected: OsString) {
		if let Some(p) = ARGS.chooser_file.clone() {
			std::fs::write(p, selected.as_encoded_bytes()).ok();
		}
	}
}
