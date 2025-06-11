use std::{borrow::Cow, time::Duration};

use tokio::pin;
use tokio_stream::{StreamExt, wrappers::UnboundedReceiverStream};
use yazi_config::popup::InputCfg;
use yazi_fs::FilterCase;
use yazi_macro::emit;
use yazi_proxy::InputProxy;
use yazi_shared::{Debounce, errors::InputError, event::{Cmd, CmdCow}};

use crate::tab::Tab;

pub(super) struct Opt {
	pub(super) query: Option<Cow<'static, str>>,
	pub(super) prev:  bool,
	pub(super) case:  FilterCase,
}

impl From<CmdCow> for Opt {
	fn from(mut c: CmdCow) -> Self {
		Self { query: c.take_first_str(), prev: c.bool("previous"), case: FilterCase::from(&*c) }
	}
}

impl Tab {
	#[yazi_codegen::command]
	pub fn find(&mut self, opt: Opt) {
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
