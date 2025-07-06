use std::time::Duration;

use tokio::pin;
use tokio_stream::{StreamExt, wrappers::UnboundedReceiverStream};
use yazi_config::popup::InputCfg;
use yazi_fs::FilterCase;
use yazi_macro::emit;
use yazi_parser::tab::FindOpt;
use yazi_proxy::InputProxy;
use yazi_shared::{Debounce, errors::InputError, event::Cmd};

use crate::tab::Tab;

impl Tab {
	#[yazi_codegen::command]
	pub fn find(&mut self, opt: FindOpt) {
		let input = InputProxy::show(InputCfg::find(opt.prev));

		tokio::spawn(async move {
			let rx = Debounce::new(UnboundedReceiverStream::new(input), Duration::from_millis(50));
			pin!(rx);

			while let Some(Ok(s)) | Some(Err(InputError::Typed(s))) = rx.next().await {
				emit!(Call(
					Cmd::args("mgr:find_do", [s])
						.with("previous", opt.prev)
						.with("smart", opt.case == FilterCase::Smart)
						.with("insensitive", opt.case == FilterCase::Insensitive)
				));
			}
		});
	}
}
