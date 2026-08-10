use anyhow::Result;
use yazi_macro::succ;
use yazi_parser::tasks::OutputForm;
use yazi_shared::data::Data;

use crate::{Actor, Ctx};

pub struct Output;

impl Actor for Output {
	type Form = OutputForm;

	const NAME: &str = "output";

	fn act(cx: &mut Ctx, form: Self::Form) -> Result<Data> {
		cx.tasks.scheduler.custom_output(form.id, form.out);
		succ!()
	}
}
