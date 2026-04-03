use anyhow::Result;
use yazi_macro::{render, succ};
use yazi_parser::help::ToggleForm;
use yazi_shared::data::Data;

use crate::{Actor, Ctx};

pub struct Toggle;

impl Actor for Toggle {
	type Form = ToggleForm;

	const NAME: &str = "toggle";

	fn act(cx: &mut Ctx, form: Self::Form) -> Result<Data> {
		let help = &mut cx.help;

		help.visible = !help.visible;
		help.layer = form.layer;

		help.keyword = String::new();
		help.in_filter = None;
		help.filter_apply();

		help.offset = 0;
		help.cursor = 0;
		succ!(render!());
	}
}
