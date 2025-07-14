use std::iter;

use anyhow::Result;
use yazi_macro::succ;
use yazi_parser::VoidOpt;
use yazi_shared::event::Data;

use crate::{Actor, Ctx};

pub struct Watch;

impl Actor for Watch {
	type Options = VoidOpt;

	const NAME: &'static str = "watch";

	fn act(cx: &mut Ctx, _: Self::Options) -> Result<Data> {
		let it = iter::once(cx.core.mgr.tabs.active().cwd())
			.chain(cx.core.mgr.tabs.parent().map(|p| &p.url))
			.chain(cx.core.mgr.tabs.hovered().filter(|h| h.is_dir()).map(|h| &h.url));

		cx.core.mgr.watcher.watch(it);
		succ!();
	}
}
