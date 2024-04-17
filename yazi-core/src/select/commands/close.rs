use anyhow::anyhow;
use yazi_shared::{event::Cmd, render};

use crate::select::Select;

pub struct Opt {
	submit: bool,
}

impl From<Cmd> for Opt {
	fn from(c: Cmd) -> Self { Self { submit: c.get_bool("submit") } }
}
impl From<bool> for Opt {
	fn from(submit: bool) -> Self { Self { submit } }
}

impl Select {
	pub fn close(&mut self, opt: impl Into<Opt>) {
		let opt = opt.into() as Opt;
		if let Some(cb) = self.callback.take() {
			_ = cb.send(if opt.submit { Ok(self.cursor) } else { Err(anyhow!("canceled")) });
		}

		self.cursor = 0;
		self.offset = 0;
		self.visible = false;
		render!();
	}
}
