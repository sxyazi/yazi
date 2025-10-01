use anyhow::Result;
use yazi_macro::{render, succ};
use yazi_parser::ArrowOpt;
use yazi_shared::data::Data;
use yazi_widgets::Scrollable;

use crate::{Actor, Ctx};

pub struct Arrow;

impl Actor for Arrow {
	type Options = ArrowOpt;

	const NAME: &str = "arrow";

	fn act(cx: &mut Ctx, opt: Self::Options) -> Result<Data> {
		succ!(render!(cx.help.scroll(opt.step)));
	}
}
