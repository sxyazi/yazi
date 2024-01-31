use yazi_plugin::external::{self, FzfOpt, ZoxideOpt};
use yazi_scheduler::{Scheduler, BLOCKER};
use yazi_shared::{event::Cmd, fs::ends_with_slash, Defer};

use crate::tab::Tab;

pub struct Opt {
	type_: OptType,
}

#[derive(PartialEq, Eq)]
pub enum OptType {
	None,
	Fzf,
	Zoxide,
}

impl From<Cmd> for Opt {
	fn from(c: Cmd) -> Self {
		Self {
			type_: match c.args.first().map(|s| s.as_str()) {
				Some("fzf") => OptType::Fzf,
				Some("zoxide") => OptType::Zoxide,
				_ => OptType::None,
			},
		}
	}
}

impl Tab {
	pub fn jump(&self, opt: impl Into<Opt>) {
		let opt = opt.into() as Opt;
		if opt.type_ == OptType::None {
			return;
		}

		let cwd = self.current.cwd.clone();
		tokio::spawn(async move {
			let _guard = BLOCKER.acquire().await.unwrap();
			let _defer = Defer::new(Scheduler::app_resume);
			Scheduler::app_stop().await;

			let result = if opt.type_ == OptType::Fzf {
				external::fzf(FzfOpt { cwd }).await
			} else {
				external::zoxide(ZoxideOpt { cwd }).await
			};

			let Ok(url) = result else {
				return;
			};

			if opt.type_ == OptType::Fzf && !ends_with_slash(&url) {
				Tab::_reveal(&url)
			} else {
				Tab::_cd(&url)
			}
		});
	}
}
