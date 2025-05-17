use std::ffi::OsString;

use tokio::fs;
use yazi_boot::ARGS;
use yazi_shared::event::EventQuit;

use crate::{Term, app::App};

impl App {
	pub(crate) fn quit(&mut self, opt: EventQuit) -> ! {
		self.cx.tasks.shutdown();
		self.cx.mgr.shutdown();

		futures::executor::block_on(async {
			_ = futures::join!(
				yazi_dds::shutdown(),
				yazi_dds::STATE.drain(),
				self.cwd_to_file(opt.no_cwd_file),
				self.selected_to_file(opt.selected)
			);
		});

		Term::goodbye(|| opt.code);
	}

	async fn cwd_to_file(&self, no: bool) {
		if let Some(p) = ARGS.cwd_file.as_ref().filter(|_| !no) {
			let cwd = self.cx.mgr.cwd().as_os_str();
			fs::write(p, cwd.as_encoded_bytes()).await.ok();
		}
	}

	async fn selected_to_file(&self, selected: Option<OsString>) {
		if let (Some(s), Some(p)) = (selected, &ARGS.chooser_file) {
			fs::write(p, s.as_encoded_bytes()).await.ok();
		}
	}
}
