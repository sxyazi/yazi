use std::time::Duration;

use anyhow::Result;
use tokio::pin;
use tokio_stream::{StreamExt, wrappers::UnboundedReceiverStream};
use yazi_config::popup::InputCfg;
use yazi_fs::FilterCase;
use yazi_macro::{emit, succ};
use yazi_parser::tab::FindOpt;
use yazi_proxy::InputProxy;
use yazi_shared::{Debounce, errors::InputError, event::{Cmd, Data}};

use crate::{Actor, Ctx};

pub struct Find;

impl Actor for Find {
	type Options = FindOpt;

	const NAME: &'static str = "find";

	fn act(_: &mut Ctx, opt: Self::Options) -> Result<Data> {
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
		succ!();
	}
}
