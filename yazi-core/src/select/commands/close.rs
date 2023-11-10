use anyhow::anyhow;
use yazi_config::keymap::Exec;

use crate::select::Select;

pub struct Opt {
	submit: bool,
}

impl From<&Exec> for Opt {
	fn from(e: &Exec) -> Self { Self { submit: e.named.contains_key("submit") } }
}
impl From<bool> for Opt {
	fn from(submit: bool) -> Self { Self { submit } }
}

impl Select {
	pub fn close(&mut self, opt: impl Into<Opt>) -> bool {
		let opt = opt.into() as Opt;
		if let Some(cb) = self.callback.take() {
			_ = cb.send(if opt.submit { Ok(self.cursor) } else { Err(anyhow!("canceled")) });
		}

		self.cursor = 0;
		self.offset = 0;
		self.visible = false;
		true
	}
}
