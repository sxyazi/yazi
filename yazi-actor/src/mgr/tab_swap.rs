use anyhow::Result;
use yazi_macro::{act, render, succ};
use yazi_parser::ArrowOpt;
use yazi_shared::event::Data;

use crate::{Actor, Ctx};

pub struct TabSwap;

impl Actor for TabSwap {
	type Options = ArrowOpt;

	const NAME: &'static str = "tab_swap";

	fn act(cx: &mut Ctx, opt: Self::Options) -> Result<Data> {
		let tabs = cx.tabs_mut();

		let new = opt.step.add(tabs.cursor, tabs.len(), 0);
		if new == tabs.cursor {
			succ!();
		}

		tabs.items.swap(tabs.cursor, new);
		tabs.set_idx(new);

		let cx = &mut Ctx::active(cx.core);
		act!(mgr:refresh, cx)?;
		act!(mgr:peek, cx, true)?;

		succ!(render!());
	}
}
