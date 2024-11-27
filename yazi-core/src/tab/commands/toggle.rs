use yazi_macro::render_and;
use yazi_proxy::AppProxy;
use yazi_shared::event::CmdCow;

use crate::tab::Tab;

struct Opt {
	state: Option<bool>,
}

impl From<CmdCow> for Opt {
	fn from(mut c: CmdCow) -> Self {
		Self {
			state: match c.take_first_str().as_deref() {
				Some("on") => Some(true),
				Some("off") => Some(false),
				_ => None,
			},
		}
	}
}

impl Tab {
	#[yazi_codegen::command]
	pub fn toggle(&mut self, opt: Opt) {
		let Some(url) = self.current.hovered().map(|h| &h.url) else {
			return;
		};

		let b = match opt.state {
			Some(true) => render_and!(self.selected.add(url)),
			Some(false) => render_and!(self.selected.remove(url)) | true,
			None => render_and!(self.selected.remove(url) || self.selected.add(url)),
		};

		if !b {
			AppProxy::notify_warn(
				"Toggle",
				"This file cannot be selected, due to path nesting conflict.",
			);
		}
	}
}
