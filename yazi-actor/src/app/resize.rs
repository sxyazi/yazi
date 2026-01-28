use anyhow::Result;
use yazi_actor::Ctx;
use yazi_macro::act;
use yazi_parser::app::ReflowOpt;
use yazi_shared::data::Data;

use crate::Actor;

pub struct Resize;

impl Actor for Resize {
	type Options = ReflowOpt;

	const NAME: &str = "resize";

	fn act(cx: &mut Ctx, opt: Self::Options) -> Result<Data> {
		act!(app:reflow, cx, opt)?;

		cx.core.current_mut().arrow(0);
		cx.core.parent_mut().map(|f| f.arrow(0));
		cx.core.current_mut().sync_page(true);

		act!(mgr:peek, cx)
	}
}
