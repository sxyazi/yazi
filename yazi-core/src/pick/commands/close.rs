use anyhow::anyhow;
use yazi_macro::render;
use yazi_shared::event::CmdCow;

use crate::pick::Pick;

struct Opt {
	submit: bool,
}

impl From<CmdCow> for Opt {
	fn from(c: CmdCow) -> Self { Self { submit: c.bool("submit") } }
}
impl From<bool> for Opt {
	fn from(submit: bool) -> Self { Self { submit } }
}

impl Pick {
	#[yazi_codegen::command]
	pub fn close(&mut self, opt: Opt) {
		if let Some(cb) = self.callback.take() {
			_ = cb.send(if opt.submit { Ok(self.cursor) } else { Err(anyhow!("canceled")) });
		}

		self.cursor = 0;
		self.offset = 0;
		self.visible = false;
		render!();
	}
}
