use std::time::Duration;

use tokio::{select, time};
use yazi_config::popup::ConfirmCfg;
use yazi_macro::emit;
use yazi_proxy::ConfirmProxy;
use yazi_shared::event::{CmdCow, EventQuit};

use crate::{mgr::Mgr, tasks::Tasks};

#[derive(Default)]
pub(super) struct Opt {
	pub(super) no_cwd_file: bool,
}
impl From<CmdCow> for Opt {
	fn from(c: CmdCow) -> Self { Self { no_cwd_file: c.bool("no-cwd-file") } }
}

impl Mgr {
	#[yazi_codegen::command]
	pub fn quit(&self, opt: Opt, tasks: &Tasks) {
		let opt = EventQuit { no_cwd_file: opt.no_cwd_file, ..Default::default() };

		let ongoing = tasks.ongoing().clone();
		let (left, left_names) = {
			let ongoing = ongoing.lock();
			(ongoing.len(), ongoing.values().take(11).map(|t| t.name.clone()).collect())
		};

		if left == 0 {
			emit!(Quit(opt));
			return;
		}

		tokio::spawn(async move {
			let mut i = 0;
			let mut rx = ConfirmProxy::show_rx(ConfirmCfg::quit(left, left_names));
			loop {
				select! {
					_ = time::sleep(Duration::from_millis(50)) => {
						i += 1;
						if i > 40 { break }
						else if ongoing.lock().is_empty() {
							emit!(Quit(opt));
							return;
						}
					}
					b = &mut rx => {
						if b.unwrap_or(false) {
							emit!(Quit(opt));
						}
						return;
					}
				}
			}

			if rx.await.unwrap_or(false) {
				emit!(Quit(opt));
			}
		});
	}
}
