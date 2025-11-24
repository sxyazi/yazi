use anyhow::Result;
use yazi_macro::{render, succ};
use yazi_parser::pick::SelectOpt;
use yazi_shared::data::Data;

use crate::{Actor, Ctx};

pub struct Select;

impl Actor for Select {
	type Options = SelectOpt;

	const NAME: &str = "select";

	fn act(cx: &mut Ctx, opt: Self::Options) -> Result<Data> {
		let pick = &mut cx.pick;
		if opt.index >= pick.items.len() {
			succ!();
		}

		if let Some(cb) = pick.callback.take() {
			_ = cb.send(Ok(opt.index));
		}

		pick.cursor = 0;
		pick.offset = 0;
		pick.visible = false;

		succ!(render!());
	}
}
