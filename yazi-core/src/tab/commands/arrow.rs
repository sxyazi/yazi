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
		// TODO: remove this
		if let Step::Fixed(n) = opt.step {
			if n <= -999999 || n >= 999999 {
				yazi_proxy::AppProxy::notify_warn(
					"Deprecated command",
					"`arrow -99999999` and `arrow 99999999` have been deprecated, please use `arrow top` and `arrow bot` instead.

See #2294 for more details: https://github.com/sxyazi/yazi/pull/2294",
				);
			}
		}

		if !self.current.arrow(opt.step) {
			return;
		}

		// Visual selection
		if let Some((start, items)) = self.mode.visual_mut() {
			let after = self.current.cursor;

			items.clear();
			for i in start.min(after)..=after.max(start) {
				items.insert(i);
			}
		}

		MgrProxy::hover(None, self.id);
		render!();
	}
}
