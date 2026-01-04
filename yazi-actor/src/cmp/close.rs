use std::mem;

use anyhow::Result;
use yazi_macro::{act, render, succ};
use yazi_parser::{cmp::CloseOpt, input::CompleteOpt};
use yazi_shared::data::Data;

use crate::{Actor, Ctx};

pub struct Close;

impl Actor for Close {
	type Options = CloseOpt;

	const NAME: &str = "close";

	fn act(cx: &mut Ctx, opt: Self::Options) -> Result<Data> {
		let cmp = &mut cx.core.cmp;
		if let Some(item) = cmp.selected().filter(|_| opt.submit).cloned() {
			return act!(input:complete, cx, CompleteOpt { item, ticket: cmp.ticket });
		}

		cmp.caches.clear();
		cmp.ticket = Default::default();
		succ!(render!(mem::replace(&mut cmp.visible, false)));
	}
}
