use anyhow::Result;
use yazi_config::ARGS;
use yazi_shared::term::Term;

use crate::app::App;

pub struct Opt {
	no_cwd_file: bool,
}

impl From<bool> for Opt {
	fn from(no_cwd_file: bool) -> Self { Self { no_cwd_file } }
}

impl App {
	pub(crate) fn quit(&mut self, opt: impl Into<Opt>) -> Result<()> {
		let opt = opt.into() as Opt;

		if let Some(p) = ARGS.cwd_file.as_ref().filter(|_| !opt.no_cwd_file) {
			let cwd = self.cx.manager.cwd().as_os_str();
			std::fs::write(p, cwd.as_encoded_bytes()).ok();
		}

		Term::goodbye(|| false);
	}
}
