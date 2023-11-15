use yazi_config::keymap::Exec;

use crate::{emit, manager::Manager, tasks::Tasks};

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
			let mut result = emit!(Input(InputOpt::top_center(
				format!("{tasks} tasks running, sure to quit? (y/N)",),
				Default::default()
			)));

			if let Some(Ok(choice)) = result.recv().await {
				if choice == "y" || choice == "Y" {
					emit!(Quit(opt.no_cwd_file));
				}
			}
		});
		false
	}
}
