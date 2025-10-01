use std::time::Duration;

use anyhow::Result;
use tokio::pin;
use tokio_stream::{StreamExt, wrappers::UnboundedReceiverStream};
use yazi_config::popup::InputCfg;
use yazi_macro::succ;
use yazi_parser::mgr::{FindDoOpt, FindOpt};
use yazi_proxy::{InputProxy, MgrProxy};
use yazi_shared::{Debounce, data::Data, errors::InputError};

use crate::{Actor, Ctx};

pub struct Find;

impl Actor for Find {
	type Options = FindOpt;

	const NAME: &str = "find";

	fn act(_: &mut Ctx, opt: Self::Options) -> Result<Data> {
		let input = InputProxy::show(InputCfg::find(opt.prev));

		tokio::spawn(async move {
			let rx = Debounce::new(UnboundedReceiverStream::new(input), Duration::from_millis(50));
			pin!(rx);

			while let Some(Ok(s)) | Some(Err(InputError::Typed(s))) = rx.next().await {
				MgrProxy::find_do(FindDoOpt { query: s.into(), prev: opt.prev, case: opt.case });
			}
		});
		succ!();
	}
}
