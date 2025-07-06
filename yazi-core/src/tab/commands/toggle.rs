use yazi_macro::render_and;
use yazi_parser::tab::ToggleOpt;
use yazi_proxy::AppProxy;

use crate::tab::Tab;

impl Tab {
	#[yazi_codegen::command]
	pub fn toggle(&mut self, opt: ToggleOpt) {
		let Some(url) = opt.url.as_ref().or(self.current.hovered().map(|h| &h.url)) else {
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
