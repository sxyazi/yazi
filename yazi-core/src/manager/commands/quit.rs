use yazi_config::popup::InputCfg;
use yazi_shared::Exec;

use crate::{emit, input::Input, manager::Manager, tasks::Tasks};

#[derive(Default)]
pub struct Opt {
	no_cwd_file: bool,
}
impl From<()> for Opt {
	fn from(_: ()) -> Self { Self::default() }
}
impl From<&Exec> for Opt {
	fn from(e: &Exec) -> Self { Self { no_cwd_file: e.named.contains_key("no-cwd-file") } }
}

impl Manager {
	pub fn quit(&self, opt: impl Into<Opt>, tasks: &Tasks) -> bool {
		let opt = opt.into() as Opt;

		let tasks = tasks.len();
		if tasks == 0 {
			emit!(Quit(opt.no_cwd_file));
			return false;
		}

		tokio::spawn(async move {
			let mut result = Input::_show(InputCfg::quit(tasks));
			if let Some(Ok(choice)) = result.recv().await {
				if choice == "y" || choice == "Y" {
					emit!(Quit(opt.no_cwd_file));
				}
			}
		});
		false
	}
}
