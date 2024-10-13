use std::time::Duration;

use yazi_shared::event::Cmd;

use crate::tab::Tab;

struct Opt;

impl From<Cmd> for Opt {
	fn from(_: Cmd) -> Self { Self }
}

impl From<()> for Opt {
	fn from(_: ()) -> Self { Self }
}

impl Tab {
	// TODO: remove this in Yazi 0.4.1
	#[yazi_codegen::command]
	pub fn select(&mut self, _: Opt) {
		yazi_proxy::AppProxy::notify(yazi_proxy::options::NotifyOpt {
			title:   "Deprecated command".to_owned(),
			content: "`select` and `select_all` command has been renamed to `toggle` and `toggle_all` in Yazi v0.4

Please change it in your keymap.toml, see #1773 for details: https://github.com/sxyazi/yazi/pull/1773".to_owned(),
			level:   yazi_proxy::options::NotifyLevel::Error,
			timeout: Duration::from_secs(20),
		});
	}
}
