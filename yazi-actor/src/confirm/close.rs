use anyhow::Result;
use yazi_macro::{render, succ};
use yazi_parser::confirm::CloseOpt;
use yazi_shared::data::Data;

use crate::{Actor, Ctx};

pub struct Close;

impl Actor for Close {
	type Options = CloseOpt;

	const NAME: &str = "close";

	fn act(cx: &mut Ctx, opt: Self::Options) -> Result<Data> {
		cx.confirm.token.complete(opt.submit);
		cx.confirm.visible = false;
		succ!(render!());
	}
}
