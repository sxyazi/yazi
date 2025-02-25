use yazi_macro::render;
use yazi_proxy::CmpProxy;
use yazi_shared::{errors::InputError, event::CmdCow};

use crate::input::Input;

struct Opt {
	submit: bool,
}

impl From<CmdCow> for Opt {
	fn from(c: CmdCow) -> Self { Self { submit: c.bool("submit") } }
}
impl From<bool> for Opt {
	fn from(submit: bool) -> Self { Self { submit } }
}

impl Input {
	#[yazi_codegen::command]
	pub fn close(&mut self, opt: Opt) {
		if self.completion {
			CmpProxy::close();
		}

		if let Some(cb) = self.callback.take() {
			let value = self.snap_mut().value.clone();
			_ = cb.send(if opt.submit { Ok(value) } else { Err(InputError::Canceled(value)) });
		}

		self.ticket = self.ticket.wrapping_add(1);
		self.visible = false;
		render!();
	}
}
