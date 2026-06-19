use std::ops::DerefMut;

use anyhow::Result;
use ratatui_core::layout::Margin;
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

		let area = cx.mgr.area(opt.position);
		let input = &mut cx.input;
		input.main.visible = true;
		input.main.title = opt.title.clone();
		input.main.position = opt.position;

		opt.styles = (&THEME.input).into();
		opt.blinking = YAZI.input.cursor_blink;
		*input.main.deref_mut() = yazi_widgets::input::Input::new(opt)?;
		input.main.repos(area.inner(Margin::new(1, 1)));

		succ!(render!());
	}
}
