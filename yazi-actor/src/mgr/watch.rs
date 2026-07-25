use std::iter;

use anyhow::Result;
use yazi_macro::succ;
use yazi_parser::VoidForm;
use yazi_shared::data::Data;

use crate::{Actor, Ctx};

pub struct Watch;

impl Actor for Watch {
	type Form = VoidForm;

	const NAME: &str = "watch";

	fn act(cx: &mut Ctx, _: Self::Form) -> Result<Data> {
		let tab = cx.core.mgr.tabs.active();

		let it = iter::once(&tab.current.file)
			.chain(tab.hovered_folder().map(|h| &h.file).or(tab.hovered().filter(|f| f.is_dir())))
			.chain(tab.parent.as_ref().map(|p| &p.file));

		cx.core.mgr.watcher.watch(it);
		succ!();
	}
}
