use anyhow::Result;
use yazi_shared::data::Data;
use yazi_widgets::input::parser::HistoryOpt;

use crate::{Actor, Ctx};

pub struct History;

impl Actor for History {
	type Form = HistoryOpt;

	const NAME: &str = "history";

	fn act(cx: &mut Ctx, form: Self::Form) -> Result<Data> {
		cx.input.navigate_history(form)
	}
}