use std::time::Duration;

use yazi_proxy::{options::{NotifyLevel, NotifyOpt}, AppProxy};
use yazi_shared::{emit, event::Cmd, Layer};

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
	fn from(mut c: Cmd) -> Self {
		Self {
			type_: match c.take_first_str().as_deref() {
				Some("fzf") => OptType::Fzf,
				Some("zoxide") => OptType::Zoxide,
				_ => OptType::None,
			},
		}
	}
}

impl Tab {
	// TODO: Remove this once Yazi v0.2.7 is released
	pub fn jump(&self, opt: impl Into<Opt>) {
		AppProxy::notify(NotifyOpt {
				title:   "Jump".to_owned(),
				content: r#"The `jump` command has been deprecated in Yazi v0.2.5.
Please replace `jump fzf` with `plugin fzf`, and `jump zoxide` with `plugin zoxide`, in your `keymap.toml`.

See https://github.com/sxyazi/yazi/issues/865 for more details."#.to_owned(),
				level:   NotifyLevel::Warn,
				timeout: Duration::from_secs(15),
			});

		let opt = opt.into() as Opt;
		if opt.type_ == OptType::None {
			return;
		}

		if opt.type_ == OptType::Fzf {
			emit!(Call(Cmd::args("plugin", vec!["fzf".to_owned()]), Layer::App));
		} else {
			emit!(Call(Cmd::args("plugin", vec!["zoxide".to_owned()]), Layer::App));
		}
	}
}
