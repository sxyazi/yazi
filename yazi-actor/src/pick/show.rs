use anyhow::Result;
use yazi_macro::{act, render, succ};
use yazi_parser::pick::ShowForm;
use yazi_shared::data::Data;

use crate::{Actor, Ctx};

pub struct Show;

impl Actor for Show {
	type Form = ShowForm;

	const NAME: &str = "show";

	fn act(cx: &mut Ctx, form: Self::Form) -> Result<Data> {
		act!(pick:close, cx)?;

		let pick = &mut cx.pick;
		pick.title = form.cfg.title;
		pick.items = form.cfg.items;
		pick.position = form.cfg.position;

		pick.callback = Some(form.tx);
		pick.visible = true;
		succ!(render!());
	}
}
