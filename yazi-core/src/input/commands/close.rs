use yazi_macro::render;
use yazi_parser::input::CloseOpt;
use yazi_proxy::CmpProxy;
use yazi_shared::errors::InputError;

use crate::input::Input;

impl Input {
	#[yazi_codegen::command]
	pub fn close(&mut self, opt: CloseOpt) {
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
