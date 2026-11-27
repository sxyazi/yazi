use anyhow::Result;
use yazi_core::Ctx;
use yazi_macro::{render, succ};
use yazi_parser::pick::ShowForm;
use yazi_shared::data::Data;
use yazi_widgets::Scrollable;

use crate::{Actor, act};

pub struct Show;

impl Actor for Show {
	type Form = ShowForm;

	const NAME: &str = "show";

	fn act(cx: &mut Ctx, form: Self::Form) -> Result<Data> {
		if form.cfg.items.is_empty() {
			succ!();
		}

		act!(pick:close, cx)?;

		let pick = &mut cx.pick;
		pick.title = form.cfg.title;
		pick.items = form.cfg.items;
		pick.position = form.cfg.position;

		pick.cursor = form.cfg.cursor.min(pick.items.len().saturating_sub(1));
		let max = pick.total().saturating_sub(pick.limit());
		pick.offset = pick.cursor.saturating_sub(pick.limit() / 2).min(max);

		pick.callback = Some(form.tx);
		pick.visible = true;
		succ!(render!());
	}
}
