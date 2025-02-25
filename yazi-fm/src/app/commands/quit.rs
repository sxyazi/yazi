use std::ffi::OsString;

use yazi_boot::ARGS;
use yazi_shared::event::EventQuit;

use crate::{Term, app::App};

impl App {
	pub(crate) fn quit(&mut self, opt: EventQuit) -> ! {
		self.cx.tasks.shutdown();
		self.cx.mgr.shutdown();
		futures::executor::block_on(yazi_dds::shutdown());
		futures::executor::block_on(yazi_dds::STATE.drain()).ok();

		if !opt.no_cwd_file {
			self.cwd_to_file();
		}
		if let Some(selected) = opt.selected {
			self.selected_to_file(selected);
		}

		Term::goodbye(|| false);
	}

	fn cwd_to_file(&self) {
		if let Some(p) = &ARGS.cwd_file {
			let cwd = self.cx.mgr.cwd().as_os_str();
			std::fs::write(p, cwd.as_encoded_bytes()).ok();
		}
	}

	fn selected_to_file(&self, selected: OsString) {
		if let Some(p) = &ARGS.chooser_file {
			std::fs::write(p, selected.as_encoded_bytes()).ok();
		}
	}
}
