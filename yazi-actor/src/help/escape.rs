use anyhow::Result;
use yazi_macro::{act, render, succ};
use yazi_parser::VoidOpt;
use yazi_shared::data::Data;

use crate::{Actor, Ctx};

pub struct Escape;

impl Actor for Escape {
	type Options = VoidOpt;

	const NAME: &str = "escape";

	fn act(cx: &mut Ctx, _: Self::Options) -> Result<Data> {
		if cx.help.keyword().is_none() {
			return act!(help:toggle, cx, cx.help.layer);
		}

		let help = &mut cx.help;
		help.keyword = String::new();
		help.in_filter = None;
		help.filter_apply();

		succ!(render!());
	}
}
