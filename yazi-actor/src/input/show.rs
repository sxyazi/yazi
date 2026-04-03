use std::ops::DerefMut;

use anyhow::Result;
use yazi_macro::{act, render, succ};
use yazi_shared::data::Data;
use yazi_widgets::input::InputOpt;

use crate::{Actor, Ctx};

pub struct Show;

impl Actor for Show {
	type Form = InputOpt;

	const NAME: &str = "show";

	fn act(cx: &mut Ctx, form: Self::Form) -> Result<Data> {
		act!(input:close, cx)?;

		let input = &mut cx.input;
		input.visible = true;
		input.title = form.cfg.title.clone();
		input.position = form.cfg.position;
		*input.deref_mut() = yazi_widgets::input::Input::new(form)?;

		succ!(render!());
	}
}
