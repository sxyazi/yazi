use anyhow::Result;
use yazi_macro::succ;
use yazi_parser::mgr::LoadHistoryForm;
use yazi_shared::data::Data;

use crate::{Actor, Ctx};

pub struct LoadHistory;

impl Actor for LoadHistory {
	type Form = LoadHistoryForm;

	const NAME: &str = "load_history";

	fn act(cx: &mut Ctx, form: Self::Form) -> Result<Data> {
		cx.input.histories.load(form.entries);
		succ!();
	}
}
