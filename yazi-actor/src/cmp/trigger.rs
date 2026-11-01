use std::mem;

use anyhow::Result;
use yazi_core::{Ctx, cmp::CmpOpt};
use yazi_macro::{render, succ};
use yazi_parser::cmp::TriggerForm;
use yazi_proxy::CmpProxy;
use yazi_shared::{data::Data, natsort, strand::StrandLike};
use yazi_vfs::Completer;

use crate::{Actor, act};

pub struct Trigger;

impl Actor for Trigger {
	type Form = TriggerForm;

	const NAME: &str = "trigger";

	fn act(cx: &mut Ctx, form: Self::Form) -> Result<Data> {
		if form.ticket.is_some_and(|t| t != cx.cmp.ticket) {
			succ!();
		} else if form.ticket.is_none() {
			cx.cmp.ticket = cx.input.lock().map(|g| g.ticket.current()).unwrap_or_default();
		}

		cx.cmp.handle.take().map(|h| h.abort());
		let Ok((cache_key, word)) = Completer::new(&*form.word).parse() else {
			return act!(cmp:close, cx, false);
		};

		let mut opt = CmpOpt { cache: None, cache_key, word, ticket: cx.cmp.ticket };
		if cx.cmp.caches.contains_key(&opt.cache_key) {
			return act!(cmp:show, cx, opt);
		}

		cx.cmp.handle = Some(tokio::spawn(async move {
			let mut cache = Completer::complete(&opt.cache_key, "").await?;
			cache.sort_unstable_by(|a, b| natsort(a.name.encoded_bytes(), b.name.encoded_bytes(), true));

			opt.cache = Some(cache);
			CmpProxy::show(opt);

			Ok(())
		}));

		succ!(render!(mem::replace(&mut cx.cmp.visible, false)));
	}
}
