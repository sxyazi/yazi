use anyhow::Result;
use yazi_macro::{render, succ};
use yazi_parser::ArrowForm;
use yazi_shared::data::Data;
use yazi_widgets::Scrollable;

use crate::{Actor, Ctx};

pub struct Arrow;

impl Actor for Arrow {
	type Form = ArrowForm;

	const NAME: &str = "arrow";

	fn act(cx: &mut Ctx, form: Self::Form) -> Result<Data> {
		succ!(render!(cx.cmp.scroll(form.step)));
	}
}
