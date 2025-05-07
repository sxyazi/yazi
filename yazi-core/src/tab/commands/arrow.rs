use yazi_fs::Step;
use yazi_macro::render;
use yazi_proxy::MgrProxy;
use yazi_shared::event::CmdCow;

use crate::tab::Tab;

struct Opt {
	step: Step,
}

impl From<CmdCow> for Opt {
	fn from(c: CmdCow) -> Self {
		Self { step: c.first().and_then(|d| d.try_into().ok()).unwrap_or_default() }
	}
}

impl From<isize> for Opt {
	fn from(n: isize) -> Self { Self { step: n.into() } }
}

impl Tab {
	#[yazi_codegen::command]
	pub fn arrow(&mut self, opt: Opt) {
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
