use anyhow::Result;
use ratatui_core::layout::Margin;
use yazi_config::popup::Help;
use yazi_macro::{render, succ};
use yazi_parser::help::ToggleForm;
use yazi_shared::data::Data;
use yazi_widgets::input::Input;

use crate::{Actor, Ctx};

pub struct Toggle;

impl Actor for Toggle {
	type Form = ToggleForm;

	const NAME: &str = "toggle";

	fn act(cx: &mut Ctx, form: Self::Form) -> Result<Data> {
		let position = Help::position();
		let area = cx.mgr.area(position);
		let input_area = area.inner(Margin::new(1, 1));

		let help = &mut cx.help;
		help.visible = true;
		help.layer = form.layer;
		help.position = position;
		help.height = area.height;

		help.input = Input::default();
		help.input.repos(input_area);

		help.keyword.clear();
		help.offset = 0;
		help.cursor = 0;
		help.filter_apply();

		succ!(render!());
	}
}
