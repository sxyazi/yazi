use anyhow::Result;
use yazi_macro::{render, succ};
use yazi_parser::which::ShowOpt;
use yazi_shared::data::Data;

use crate::{Actor, Ctx};

pub struct Show;

impl Actor for Show {
	type Options = ShowOpt;

	const NAME: &str = "show";

	fn act(cx: &mut Ctx, opt: Self::Options) -> Result<Data> {
		if opt.cands.is_empty() {
			succ!();
		}

		let which = &mut cx.which;
		which.times = 0;
		which.cands = opt.cands.into_iter().map(|c| c.into()).collect();

		which.visible = true;
		which.silent = opt.silent;
		succ!(render!());
	}
}
