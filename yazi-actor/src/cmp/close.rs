use std::mem;

use anyhow::Result;
use yazi_core::Ctx;
use yazi_macro::{render, succ};
use yazi_parser::cmp::CloseForm;
use yazi_shared::data::Data;
use yazi_widgets::input::parser::CompleteOpt;

use crate::{Actor, act};

pub struct Close;

impl Actor for Close {
	type Form = CloseForm;

	const NAME: &str = "close";

	fn act(cx: &mut Ctx, form: Self::Form) -> Result<Data> {
		if form.submit && cx.cmp.visible {
			Self::flush_last_input(cx)?;
		}

		// On a cache miss, flush_last_input() spawns a query and hides the list,
		// leaving old matches until the cache is filled.
		if form.submit && !cx.cmp.visible {
			succ!();
		}

		let cmp = &mut cx.cmp;
		if let Some(item) = cmp.selected().filter(|_| form.submit).cloned() {
			return act!(input:complete, cx, CompleteOpt { item, ticket: cmp.ticket });
		}

		cmp.caches.clear();
		cmp.ticket = Default::default();
		cmp.handle.take().map(|h| h.abort());
		succ!(render!(mem::replace(&mut cmp.visible, false)));
	}
}

impl Close {
	fn flush_last_input(cx: &mut Ctx) -> Result<Data> {
		let Some(guard) = cx.input.lock() else { succ!() };
		if cx.cmp.ticket == guard.ticket.current() {
			succ!();
		}

		let before = guard.partition().0.to_owned();
		drop(guard);

		act!(cmp:trigger, cx, before)
	}
}
