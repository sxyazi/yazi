use std::time::Duration;

use tokio::pin;
use tokio_stream::{StreamExt, wrappers::UnboundedReceiverStream};
use yazi_config::popup::InputCfg;
use yazi_fs::FilterCase;
use yazi_proxy::InputProxy;
use yazi_shared::{Debounce, InputError, Layer, emit, event::Cmd};

use crate::tab::Tab;

pub struct Opt {
	pub(super) query: Option<String>,
	pub(super) prev:  bool,
	pub(super) case:  FilterCase,
}

impl From<Cmd> for Opt {
	fn from(mut c: Cmd) -> Self {
		Self { query: c.take_first_str(), prev: c.bool("previous"), case: FilterCase::from(&c) }
	}
}

impl Tab {
	#[yazi_macro::command]
	pub fn find(&mut self, opt: Opt) {
		tokio::spawn(async move {
			let rx = InputProxy::show(InputCfg::find(opt.prev));

			let rx = Debounce::new(UnboundedReceiverStream::new(rx), Duration::from_millis(50));
			pin!(rx);

			while let Some(Ok(s)) | Some(Err(InputError::Typed(s))) = rx.next().await {
				emit!(Call(
					Cmd::args("find_do", &[s])
						.with_bool("previous", opt.prev)
						.with_bool("smart", opt.case == FilterCase::Smart)
						.with_bool("insensitive", opt.case == FilterCase::Insensitive),
					Layer::Manager
				));
			}
		});
	}
}
