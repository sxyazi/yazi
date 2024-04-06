use std::time::Duration;

use yazi_plugin::external::{self, FzfOpt};
use yazi_proxy::{options::{NotifyLevel, NotifyOpt}, AppProxy, TabProxy, HIDER};
use yazi_shared::{emit, event::Cmd, fs::ends_with_slash, Defer, Layer};

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

		// TODO: Remove this once Yazi v0.2.7 is released
		if opt.type_ == OptType::Zoxide {
			AppProxy::notify(NotifyOpt {
				title:   "Jump".to_owned(),
				content: r#"The `jump zoxide` command has been deprecated in Yazi v0.2.5. Please replace it with `plugin zoxide` in your `keymap.toml`.

See https://github.com/sxyazi/yazi/issues/865 for more details."#.to_owned(),
				level:   NotifyLevel::Warn,
				timeout: Duration::from_secs(15),
			});

			emit!(Call(Cmd::args("plugin", vec!["zoxide".to_owned()]), Layer::App));
			return;
		}

		let cwd = self.current.cwd.clone();
		tokio::spawn(async move {
			let _permit = HIDER.acquire().await.unwrap();
			let _defer = Defer::new(AppProxy::resume);
			AppProxy::stop().await;

			let result = if opt.type_ == OptType::Fzf {
				external::fzf(FzfOpt { cwd }).await
			} else {
				unreachable!()
			};

			let Ok(url) = result else {
				return;
			};

			if opt.type_ == OptType::Fzf && !ends_with_slash(&url) {
				TabProxy::reveal(&url)
			} else {
				TabProxy::cd(&url)
			}
		});
	}
}
