use anyhow::Result;
use ratatui_widgets::block::Padding;
use yazi_config::{THEME, YAZI};
use yazi_macro::{act, render, succ};
use yazi_parser::input::ShowForm;
use yazi_shared::data::Data;

use crate::{Actor, Ctx};

pub struct Show;

impl Actor for Show {
	type Form = ShowForm;

	const NAME: &str = "show";

	fn act(cx: &mut Ctx, Self::Form { mut opt }: Self::Form) -> Result<Data> {
		act!(input:close, cx)?;

		let input = &mut cx.input;
		input.main_visible = true;
		input.main_title = opt.title.clone();
		input.main_position = opt.position;

		opt.styles = (&THEME.input).into();
		opt.blinking = YAZI.input.cursor_blink;
		opt.position = opt.position.padding(Padding::uniform(1));
		input.main = yazi_widgets::input::Input::new(opt)?;

		succ!(render!());
	}
}
