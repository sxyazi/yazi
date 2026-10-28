use anyhow::Result;
use yazi_core::Ctx;
use yazi_macro::{render, succ};
use yazi_parser::mgr::UpdatePeekedForm;
use yazi_shared::data::Data;

use crate::Actor;

pub struct UpdatePeeked;

impl Actor for UpdatePeeked {
	type Form = UpdatePeekedForm;

	const NAME: &str = "update_peeked";

	fn act(cx: &mut Ctx, form: Self::Form) -> Result<Data> {
		let Some(hovered) = cx.hovered_url() else {
			succ!(cx.preview_mut().reset());
		};

		if form.lock.url == *hovered && !cx.scope.is_cancelled() {
			cx.preview_mut().lock = Some(form.lock);
			render!();
		}

		succ!();
	}
}
