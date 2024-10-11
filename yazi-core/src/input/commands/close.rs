use yazi_proxy::CompletionProxy;
use yazi_shared::{InputError, event::Cmd, render};

use crate::input::Input;

pub struct Opt {
	submit: bool,
}

impl From<Cmd> for Opt {
	fn from(c: Cmd) -> Self { Self { submit: c.bool("submit") } }
}
impl From<bool> for Opt {
	fn from(submit: bool) -> Self { Self { submit } }
}

impl Input {
	#[yazi_codegen::command]
	pub fn close(&mut self, opt: Opt) {
		if self.completion {
			CompletionProxy::close();
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
