use anyhow::Result;
use yazi_macro::{act, render, succ};
use yazi_parser::confirm::ShowForm;
use yazi_shared::data::Data;

use crate::{Actor, Ctx};

pub struct Show;

impl Actor for Show {
	type Form = ShowForm;

	const NAME: &str = "show";

	fn act(cx: &mut Ctx, form: Self::Form) -> Result<Data> {
		act!(confirm:close, cx)?;

		let confirm = &mut cx.confirm;
		confirm.title = form.cfg.title;
		confirm.body = form.cfg.body;
		confirm.list = form.cfg.list;

		confirm.position = form.cfg.position;
		confirm.offset = 0;

		confirm.token = form.token;
		confirm.visible = true;

		succ!(render!());
	}
}
