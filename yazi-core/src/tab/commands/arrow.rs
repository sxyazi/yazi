use yazi_macro::render;
use yazi_parser::tab::ArrowOpt;
use yazi_proxy::MgrProxy;

use crate::tab::Tab;

impl Tab {
	#[yazi_codegen::command]
	pub fn arrow(&mut self, opt: ArrowOpt) {
		if !self.current.arrow(opt.step) {
			return;
		}

		// Visual selection
		if let Some((start, items)) = self.mode.visual_mut() {
			let end = self.current.cursor;
			*items = (start.min(end)..=end.max(start)).collect();
		}

		self.hover(None);
		MgrProxy::peek(false);
		MgrProxy::watch();

		render!();
	}
}
