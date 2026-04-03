use anyhow::Result;
use yazi_macro::{render, succ};
use yazi_parser::pick::CloseForm;
use yazi_shared::data::Data;

use crate::{Actor, Ctx};

pub struct Close;

impl Actor for Close {
	type Form = CloseForm;

	const NAME: &str = "close";

	fn act(cx: &mut Ctx, form: Self::Form) -> Result<Data> {
		let pick = &mut cx.pick;
		if let Some(cb) = pick.callback.take() {
			_ = cb.send(if form.submit { Some(pick.cursor) } else { None });
		}

		pick.cursor = 0;
		pick.offset = 0;
		pick.visible = false;
		succ!(render!());
	}
}
