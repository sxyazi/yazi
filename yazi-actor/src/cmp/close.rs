use std::mem;

use anyhow::Result;
use yazi_macro::{act, render, succ};
use yazi_parser::cmp::CloseForm;
use yazi_shared::data::Data;
use yazi_widgets::input::parser::CompleteOpt;

use crate::{Actor, Ctx};

pub struct Close;

impl Actor for Close {
	type Form = CloseForm;

	const NAME: &str = "close";

	fn act(cx: &mut Ctx, form: Self::Form) -> Result<Data> {
		let cmp = &mut cx.cmp;
		if let Some(item) = cmp.selected().filter(|_| form.submit).cloned() {
			return act!(input:complete, cx, CompleteOpt { name: item.name, is_dir: item.is_dir, ticket: cmp.ticket });
		}

		cmp.caches.clear();
		cmp.ticket = Default::default();
		cmp.handle.take().map(|h| h.abort());
		succ!(render!(mem::replace(&mut cmp.visible, false)));
	}
}
