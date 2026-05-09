use std::time::Duration;

use anyhow::Result;
use tokio::pin;
use tokio_stream::{StreamExt, wrappers::UnboundedReceiverStream};
use yazi_config::popup::InputCfg;
use yazi_core::mgr::FilterOpt;
use yazi_macro::{input, succ};
use yazi_parser::mgr::FilterForm;
use yazi_proxy::MgrProxy;
use yazi_shared::{Debounce, data::Data};
use yazi_widgets::input::InputEvent;

use crate::{Actor, Ctx};

pub struct Filter;

impl Actor for Filter {
	type Form = FilterForm;

	const NAME: &str = "filter";

	fn act(cx: &mut Ctx, Self::Form { opt }: Self::Form) -> Result<Data> {
		let input = input!(cx, InputCfg::filter())?;

		tokio::spawn(async move {
			let rx = Debounce::new(UnboundedReceiverStream::new(input), Duration::from_millis(50));
			pin!(rx);

			while let Some(event) = rx.next().await {
				let done = event.is_submit();
				let (InputEvent::Submit(s) | InputEvent::Type(s)) = event else { continue };

				MgrProxy::filter_do(FilterOpt { query: s.into(), case: opt.case, done });
			}
		});
		succ!();
	}
}
