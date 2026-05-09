use std::time::Duration;

use anyhow::Result;
use tokio::pin;
use tokio_stream::{StreamExt, wrappers::UnboundedReceiverStream};
use yazi_config::popup::InputCfg;
use yazi_core::mgr::FindDoOpt;
use yazi_macro::{input, succ};
use yazi_parser::mgr::FindForm;
use yazi_proxy::MgrProxy;
use yazi_shared::{Debounce, data::Data};
use yazi_widgets::input::InputEvent;

use crate::{Actor, Ctx};

pub struct Find;

impl Actor for Find {
	type Form = FindForm;

	const NAME: &str = "find";

	fn act(cx: &mut Ctx, form: Self::Form) -> Result<Data> {
		let input = input!(cx, InputCfg::find(form.prev))?;

		tokio::spawn(async move {
			let rx = Debounce::new(UnboundedReceiverStream::new(input), Duration::from_millis(50));
			pin!(rx);

			while let Some(InputEvent::Submit(s) | InputEvent::Type(s)) = rx.next().await {
				MgrProxy::find_do(FindDoOpt { query: s.into(), prev: form.prev, case: form.case });
			}
		});
		succ!();
	}
}
