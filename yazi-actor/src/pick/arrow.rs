use anyhow::Result;
use yazi_macro::{render, succ};
use yazi_parser::ArrowOpt;
use yazi_shared::event::Data;
use yazi_widgets::Scrollable;

use crate::{Actor, Ctx};

pub struct Arrow;

impl Actor for Arrow {
	type Options = ArrowOpt;

	const NAME: &'static str = "arrow";

	fn act(cx: &mut Ctx, opt: Self::Options) -> Result<Data> {
		succ!(render!(cx.pick.scroll(opt.step)));
	}
}
