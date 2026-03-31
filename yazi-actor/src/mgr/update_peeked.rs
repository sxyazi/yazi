use anyhow::Result;
use yazi_macro::{render, succ};
use yazi_parser::mgr::UpdatePeekedForm;
use yazi_shared::data::Data;

use crate::{Actor, Ctx};

pub struct UpdatePeeked;

impl Actor for UpdatePeeked {
	type Form = UpdatePeekedForm;

	const NAME: &str = "update_peeked";

	fn act(cx: &mut Ctx, form: Self::Form) -> Result<Data> {
		let Some(hovered) = cx.hovered().map(|h| &h.url) else {
			succ!(cx.tab_mut().preview.reset());
		};

		if form.lock.url == *hovered {
			cx.tab_mut().preview.lock = Some(form.lock);
			render!();
		}

		succ!();
	}
}
