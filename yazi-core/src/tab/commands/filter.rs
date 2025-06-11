use std::{borrow::Cow, time::Duration};

use tokio::pin;
use tokio_stream::{StreamExt, wrappers::UnboundedReceiverStream};
use yazi_config::popup::InputCfg;
use yazi_fs::FilterCase;
use yazi_macro::emit;
use yazi_proxy::InputProxy;
use yazi_shared::{Debounce, errors::InputError, event::{Cmd, CmdCow}};

use crate::tab::Tab;

#[derive(Default)]
pub(super) struct Opt {
	pub query: Cow<'static, str>,
	pub case:  FilterCase,
	pub done:  bool,
}

impl From<CmdCow> for Opt {
	fn from(mut c: CmdCow) -> Self {
		Self {
			query: c.take_first_str().unwrap_or_default(),
			case:  FilterCase::from(&*c),
			done:  c.bool("done"),
		}
	}
}

impl Tab {
	#[yazi_codegen::command]
	pub fn filter(&mut self, opt: Opt) {
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
