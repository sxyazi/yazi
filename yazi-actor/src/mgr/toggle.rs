use anyhow::Result;
use yazi_macro::{render_and, succ};
use yazi_parser::tab::ToggleOpt;
use yazi_proxy::AppProxy;
use yazi_shared::event::Data;

use crate::{Actor, Ctx};

pub struct Toggle;

impl Actor for Toggle {
	type Options = ToggleOpt;

	const NAME: &'static str = "toggle";

	fn act(cx: &mut Ctx, opt: Self::Options) -> Result<Data> {
		let tab = cx.tab_mut();
		let Some(url) = opt.url.as_ref().or(tab.current.hovered().map(|h| &h.url)) else { succ!() };

		let b = match opt.state {
			Some(true) => render_and!(tab.selected.add(url)),
			Some(false) => render_and!(tab.selected.remove(url)) | true,
			None => render_and!(tab.selected.remove(url) || tab.selected.add(url)),
		};

		if !b {
			AppProxy::notify_warn(
				"Toggle",
				"This file cannot be selected, due to path nesting conflict.",
			);
		}
		succ!();
	}
}
