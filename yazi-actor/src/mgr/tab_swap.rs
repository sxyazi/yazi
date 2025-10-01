use anyhow::Result;
use yazi_dds::Pubsub;
use yazi_macro::{err, render, succ};
use yazi_parser::ArrowOpt;
use yazi_shared::data::Data;

use crate::{Actor, Ctx};

pub struct TabSwap;

impl Actor for TabSwap {
	type Options = ArrowOpt;

	const NAME: &str = "tab_swap";

	fn act(cx: &mut Ctx, opt: Self::Options) -> Result<Data> {
		let tabs = cx.tabs_mut();

		let new = opt.step.add(tabs.cursor, tabs.len(), 0);
		if new == tabs.cursor {
			succ!();
		}

		tabs.items.swap(tabs.cursor, new);
		tabs.cursor = new;

		err!(Pubsub::pub_after_tab(cx.active().id));
		succ!(render!());
	}
}
