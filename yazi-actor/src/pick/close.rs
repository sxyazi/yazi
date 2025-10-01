use anyhow::{Result, anyhow};
use yazi_macro::{render, succ};
use yazi_parser::pick::CloseOpt;
use yazi_shared::data::Data;

use crate::{Actor, Ctx};

pub struct Close;

impl Actor for Close {
	type Options = CloseOpt;

	const NAME: &str = "close";

	fn act(cx: &mut Ctx, opt: Self::Options) -> Result<Data> {
		let pick = &mut cx.pick;
		if let Some(cb) = pick.callback.take() {
			_ = cb.send(if opt.submit { Ok(pick.cursor) } else { Err(anyhow!("canceled")) });
		}

		pick.cursor = 0;
		pick.offset = 0;
		pick.visible = false;
		succ!(render!());
	}
}
