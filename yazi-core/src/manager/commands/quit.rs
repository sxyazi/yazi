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
	fn from(c: Cmd) -> Self { Self { no_cwd_file: c.named.contains_key("no-cwd-file") } }
}

impl Manager {
	pub fn quit(&self, opt: impl Into<Opt>, tasks: &Tasks) {
		let opt = EventQuit { no_cwd_file: opt.into().no_cwd_file, ..Default::default() };

		let tasks = tasks.len();
		if tasks == 0 {
			emit!(Quit(opt));
			return;
		}

		tokio::spawn(async move {
			let mut result = InputProxy::show(InputCfg::quit(tasks));
			if let Some(Ok(choice)) = result.recv().await {
				if choice == "y" || choice == "Y" {
					emit!(Quit(opt));
				}
			}
		});
	}
}
