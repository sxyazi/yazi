use anyhow::Result;
use yazi_macro::{render, succ};
use yazi_parser::tab::UpdatePeekedOpt;
use yazi_shared::event::Data;

use crate::Actor;

pub struct UpdatePeeked;

impl Actor for UpdatePeeked {
	type Options = UpdatePeekedOpt;

	const NAME: &'static str = "update_peeked";

	fn act(cx: &mut crate::Ctx, opt: Self::Options) -> Result<Data> {
		let Some(hovered) = cx.hovered().map(|h| &h.url) else {
			succ!(cx.tab_mut().preview.reset());
		};

		if opt.lock.url == *hovered {
			cx.tab_mut().preview.lock = Some(opt.lock);
			render!();
		}

		succ!();
	}
}
