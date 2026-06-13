use anyhow::Result;
use ratatui::widgets::Padding;
use yazi_macro::{act, render, succ};
use yazi_shared::data::Data;
use yazi_widgets::input::InputOpt;

use crate::{Actor, Ctx};

pub struct Show;

impl Actor for Show {
	type Form = InputOpt;

	const NAME: &str = "show";

	fn act(cx: &mut Ctx, mut form: Self::Form) -> Result<Data> {
		act!(input:close, cx)?;

		let input = &mut cx.input;
		input.main_visible = true;
		input.main_title = form.cfg.title.clone();
		input.main_position = form.cfg.position;

		form.cfg.position = form.cfg.position.padding(Padding::uniform(1));
		input.main = yazi_widgets::input::Input::new(form)?;

		succ!(render!());
	}
}
