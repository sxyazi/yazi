use std::time::Duration;

use anyhow::Result;
use tokio::pin;
use tokio_stream::{StreamExt, wrappers::UnboundedReceiverStream};
use yazi_config::popup::InputCfg;
use yazi_macro::succ;
use yazi_parser::mgr::FilterOpt;
use yazi_proxy::{InputProxy, MgrProxy};
use yazi_shared::{Debounce, data::Data, errors::InputError};

use crate::{Actor, Ctx};

pub struct Filter;

impl Actor for Filter {
	type Options = FilterOpt;

	const NAME: &str = "filter";

	fn act(_: &mut Ctx, opt: Self::Options) -> Result<Data> {
		let input = InputProxy::show(InputCfg::filter());

		tokio::spawn(async move {
			let rx = Debounce::new(UnboundedReceiverStream::new(input), Duration::from_millis(50));
			pin!(rx);

			while let Some(result) = rx.next().await {
				let done = result.is_ok();
				let (Ok(s) | Err(InputError::Typed(s))) = result else { continue };

				MgrProxy::filter_do(FilterOpt { query: s.into(), case: opt.case, done });
			}
		});
		succ!();
	}
}
