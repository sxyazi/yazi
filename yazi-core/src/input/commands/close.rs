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
		self.visible = false;
		self.ticket.next();

		if let Some(tx) = self.tx.take() {
			let value = self.snap().value.clone();
			_ = tx.send(if opt.submit { Ok(value) } else { Err(InputError::Canceled(value)) });
		}

		CmpProxy::close();
		render!();
	}
}
