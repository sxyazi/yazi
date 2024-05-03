use std::time::Duration;

use tokio::{select, time};
use yazi_config::popup::InputCfg;
use yazi_proxy::InputProxy;
use yazi_shared::{emit, event::{Cmd, EventQuit}};

use crate::{manager::Manager, tasks::Tasks};

#[derive(Default)]
pub struct Opt {
	no_cwd_file: bool,
}
impl From<()> for Opt {
	fn from(_: ()) -> Self { Self::default() }
}
impl From<Cmd> for Opt {
	fn from(c: Cmd) -> Self { Self { no_cwd_file: c.bool("no-cwd-file") } }
}

impl Manager {
	pub fn quit(&self, opt: impl Into<Opt>, tasks: &Tasks) {
		let opt = EventQuit { no_cwd_file: opt.into().no_cwd_file, ..Default::default() };

		let ongoing = tasks.ongoing().clone();
		let left = ongoing.lock().len();

		if left == 0 {
			emit!(Quit(opt));
			return;
		}

		tokio::spawn(async move {
			let mut i = 0;
			let mut result = InputProxy::show(InputCfg::quit(left));
			loop {
				select! {
					_ = time::sleep(Duration::from_millis(100)) => {
						i += 1;
						if i > 30 { break }
						else if ongoing.lock().len() == 0 {
							emit!(Quit(opt));
							return;
						}
					}
					choice = result.recv() => {
						if matches!(choice, Some(Ok(s)) if s == "y" || s == "Y") {
							emit!(Quit(opt));
						}
						return;
					}
				}
			}

			if let Some(Ok(choice)) = result.recv().await {
				if choice == "y" || choice == "Y" {
					emit!(Quit(opt));
				}
			}
		});
	}
}
