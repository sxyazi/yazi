use anyhow::Result;
use yazi_macro::{act, render, succ};
use yazi_parser::mgr::TabCloseForm;
use yazi_shared::data::Data;

use crate::{Actor, Ctx};

pub struct TabClose;

impl Actor for TabClose {
	type Form = TabCloseForm;

	const NAME: &str = "tab_close";

	fn act(cx: &mut Ctx, form: Self::Form) -> Result<Data> {
		let len = cx.tabs().len();
		if len < 2 || form.idx >= len {
			succ!();
		}

		let tabs = cx.tabs_mut();
		tabs.remove(form.idx).shutdown();

		if form.idx > tabs.cursor {
			tabs.set_idx(tabs.cursor);
		} else {
			tabs.set_idx(usize::min(tabs.cursor + 1, tabs.len() - 1));
		}

		let cx = &mut Ctx::renew(cx);
		act!(mgr:refresh, cx)?;
		act!(mgr:peek, cx, true)?;
		act!(app:title, cx).ok();

		succ!(render!());
	}
}
