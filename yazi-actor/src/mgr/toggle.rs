use anyhow::Result;
use yazi_macro::{render_and, succ};
use yazi_parser::mgr::ToggleOpt;
use yazi_proxy::AppProxy;
use yazi_shared::{data::Data, url::UrlCow};

use crate::{Actor, Ctx};

pub struct Toggle;

impl Actor for Toggle {
	type Options = ToggleOpt;

	const NAME: &str = "toggle";

	fn act(cx: &mut Ctx, opt: Self::Options) -> Result<Data> {
		let tab = cx.tab_mut();
		let Some(url) = opt.url.or(tab.current.hovered().map(|h| UrlCow::from(&h.url))) else {
			succ!();
		};

		let b = match opt.state {
			Some(true) => render_and!(tab.selected.add(&url)),
			Some(false) => render_and!(tab.selected.remove(&url)) | true,
			None => render_and!(tab.selected.remove(&url) || tab.selected.add(&url)),
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
