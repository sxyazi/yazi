use anyhow::Result;
use yazi_macro::{render, succ};
use yazi_parser::VoidForm;
use yazi_shared::data::Data;

use crate::{Actor, Ctx};

pub struct Filter;

impl Actor for Filter {
	type Form = VoidForm;

	const NAME: &str = "filter";

	fn act(cx: &mut Ctx, _: Self::Form) -> Result<Data> {
		let help = &mut cx.help;

		help.in_filter = Some(Default::default());
		help.filter_apply();
		succ!(render!());
	}
}
