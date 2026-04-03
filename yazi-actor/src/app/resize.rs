use anyhow::Result;
use yazi_actor::Ctx;
use yazi_macro::act;
use yazi_parser::app::ReflowForm;
use yazi_shared::data::Data;

use crate::Actor;

pub struct Resize;

impl Actor for Resize {
	type Form = ReflowForm;

	const NAME: &str = "resize";

	fn act(cx: &mut Ctx, form: Self::Form) -> Result<Data> {
		act!(app:reflow, cx, form)?;

		cx.current_mut().arrow(0);
		cx.parent_mut().map(|f| f.arrow(0));
		cx.current_mut().sync_page(true);

		act!(mgr:peek, cx)
	}
}
