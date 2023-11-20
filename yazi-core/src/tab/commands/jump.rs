use yazi_config::keymap::Exec;
use yazi_shared::{ends_with_slash, Defer};

use crate::{emit, external::{self, FzfOpt, ZoxideOpt}, tab::Tab, Event, BLOCKER};

pub struct Opt {
	type_: OptType,
}

#[derive(PartialEq, Eq)]
pub enum OptType {
	None,
	Fzf,
	Zoxide,
}

impl From<&Exec> for Opt {
	fn from(e: &Exec) -> Self {
		Self {
			type_: match e.args.first().map(|s| s.as_str()) {
				Some("fzf") => OptType::Fzf,
				Some("zoxide") => OptType::Zoxide,
				_ => OptType::None,
			},
		}
	}
}

impl Tab {
	pub fn jump(&self, opt: impl Into<Opt>) -> bool {
		let opt = opt.into() as Opt;
		if opt.type_ == OptType::None {
			return false;
		}

		let cwd = self.current.cwd.clone();
		tokio::spawn(async move {
			let _guard = BLOCKER.acquire().await.unwrap();
			let _defer = Defer::new(|| Event::Stop(false, None).emit());
			emit!(Stop(true)).await;

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
		false
	}
}
