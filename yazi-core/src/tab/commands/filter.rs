use std::time::Duration;

use tokio::pin;
use tokio_stream::{StreamExt, wrappers::UnboundedReceiverStream};
use yazi_config::popup::InputCfg;
use yazi_fs::FilterCase;
use yazi_macro::emit;
use yazi_parser::tab::FilterOpt;
use yazi_proxy::InputProxy;
use yazi_shared::{Debounce, errors::InputError, event::Cmd};

use crate::tab::Tab;

impl Tab {
	#[yazi_codegen::command]
	pub fn filter(&mut self, opt: FilterOpt) {
		let input = InputProxy::show(InputCfg::filter());

		tokio::spawn(async move {
			let rx = Debounce::new(UnboundedReceiverStream::new(input), Duration::from_millis(50));
			pin!(rx);

			while let Some(result) = rx.next().await {
				let done = result.is_ok();
				let (Ok(s) | Err(InputError::Typed(s))) = result else { continue };

				emit!(Call(
					Cmd::args("mgr:filter_do", [s])
						.with("smart", opt.case == FilterCase::Smart)
						.with("insensitive", opt.case == FilterCase::Insensitive)
						.with("done", done)
				));
			}
		});
	}
}
