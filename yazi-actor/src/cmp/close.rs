use std::mem;

use anyhow::Result;
use yazi_macro::{act, render, succ};
use yazi_parser::{cmp::CloseOpt, input::CompleteOpt};
use yazi_shared::event::Data;

use crate::{Actor, Ctx};

pub struct Close;

impl Actor for Close {
	type Options = CloseOpt;

	const NAME: &'static str = "close";

	fn act(cx: &mut Ctx, opt: Self::Options) -> Result<Data> {
		let cmp = &mut cx.core.cmp;
		if let Some(item) = cmp.selected().filter(|_| opt.submit).cloned() {
			return act!(complete, cx.core.input, CompleteOpt { item, _ticket: cmp.ticket });
		}

		cmp.caches.clear();
		succ!(render!(mem::replace(&mut cmp.visible, false)));
	}
}
