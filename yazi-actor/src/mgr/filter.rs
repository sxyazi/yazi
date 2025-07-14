use std::time::Duration;

use anyhow::Result;
use tokio::pin;
use tokio_stream::{StreamExt, wrappers::UnboundedReceiverStream};
use yazi_config::popup::InputCfg;
use yazi_fs::FilterCase;
use yazi_macro::{emit, succ};
use yazi_parser::tab::FilterOpt;
use yazi_proxy::InputProxy;
use yazi_shared::{Debounce, errors::InputError, event::{Cmd, Data}};

use crate::{Actor, Ctx};

pub struct Filter;

impl Actor for Filter {
	type Options = FilterOpt;

	const NAME: &'static str = "filter";

	fn act(_: &mut Ctx, opt: Self::Options) -> Result<Data> {
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
		succ!();
	}
}
