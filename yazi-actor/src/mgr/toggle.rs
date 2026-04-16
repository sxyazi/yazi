use anyhow::Result;
use yazi_macro::{render_and, succ};
use yazi_parser::mgr::ToggleForm;
use yazi_scheduler::NotifyProxy;
use yazi_shared::data::Data;

use crate::{Actor, Ctx};

pub struct Toggle;

impl Actor for Toggle {
	type Form = ToggleForm;

	const NAME: &str = "toggle";

	fn act(cx: &mut Ctx, form: Self::Form) -> Result<Data> {
		let tab = cx.tab_mut();
		let Some(url) = form.url.or_else(|| tab.hovered().map(|h| h.url.clone())) else {
			succ!();
		};

		let b = match form.state {
			Some(true) => render_and!(tab.selected.add(&url)),
			Some(false) => render_and!(tab.selected.remove(&url)) | true,
			None => render_and!(tab.selected.remove(&url) || tab.selected.add(&url)),
		};

		if !b {
			NotifyProxy::push_warn(
				"Toggle",
				"This file cannot be selected, due to path nesting conflict.",
			);
		}
		succ!();
	}
}
