use anyhow::Result;
use yazi_dds::Pubsub;
use yazi_macro::{err, render, succ};
use yazi_parser::ArrowForm;
use yazi_shared::data::Data;

use crate::{Actor, Ctx};

pub struct TabSwap;

impl Actor for TabSwap {
	type Form = ArrowForm;

	const NAME: &str = "tab_swap";

	fn act(cx: &mut Ctx, form: Self::Form) -> Result<Data> {
		let tabs = cx.tabs_mut();

		let new = form.step.add(tabs.cursor, tabs.len(), 0);
		if new == tabs.cursor {
			succ!();
		}

		tabs.items.swap(tabs.cursor, new);
		tabs.cursor = new;

		err!(Pubsub::pub_after_tab(cx.active().id));
		succ!(render!());
	}
}
