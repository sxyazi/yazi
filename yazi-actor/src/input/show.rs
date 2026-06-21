use std::ops::DerefMut;

use anyhow::Result;
use yazi_config::{THEME, YAZI};
use yazi_macro::{act, render, succ};
use yazi_parser::input::ShowForm;
use yazi_shared::data::Data;
use yazi_shim::ratatui::Padable;

use crate::{Actor, Ctx};

pub struct Show;

impl Actor for Show {
	type Form = ShowForm;

	const NAME: &str = "show";

	fn act(cx: &mut Ctx, Self::Form { mut opt }: Self::Form) -> Result<Data> {
		act!(input:close, cx)?;

		let area = cx.mgr.area(opt.position).padding(cx.input.padding());
		let input = &mut cx.input;
		input.main.visible = true;
		input.main.title = opt.title.clone();
		input.main.position = opt.position;

		opt.styles = (&THEME.input).into();
		opt.blinking = YAZI.input.cursor_blink;
		*input.main.deref_mut() = yazi_widgets::input::Input::new(opt)?;
		input.main.repos(area);

		succ!(render!());
	}
}
