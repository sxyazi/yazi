use tracing::error;
use yazi_parser::which::CallbackOpt;

use crate::which::Which;

impl Which {
	pub fn callback(&mut self, opt: impl TryInto<CallbackOpt>) {
		let Ok(opt) = opt.try_into() else {
			return;
		};

		if opt.tx.try_send(opt.idx).is_err() {
			error!("which callback: send error");
		}
	}
}
