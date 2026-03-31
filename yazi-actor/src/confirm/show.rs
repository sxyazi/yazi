use anyhow::Result;
use yazi_macro::{act, render, succ};
use yazi_parser::confirm::ShowForm;
use yazi_shared::data::Data;

use crate::{Actor, Ctx};

pub struct Show;

impl Actor for Show {
	type Form = ShowForm;

	const NAME: &str = "show";

	fn act(cx: &mut Ctx, opt: Self::Form) -> Result<Data> {
		act!(confirm:close, cx)?;

		let confirm = &mut cx.confirm;
		confirm.title = opt.cfg.title;
		confirm.body = opt.cfg.body;
		confirm.list = opt.cfg.list;

		confirm.position = opt.cfg.position;
		confirm.offset = 0;

		confirm.token = opt.token;
		confirm.visible = true;

		succ!(render!());
	}
}
