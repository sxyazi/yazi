use anyhow::Result;
use yazi_macro::{act, render, succ};
use yazi_parser::VoidForm;
use yazi_shared::data::Data;
use yazi_widgets::input::InputMode;

use crate::{Actor, Ctx};

pub struct Escape;

impl Actor for Escape {
	type Form = VoidForm;

	const NAME: &str = "escape";

	fn act(cx: &mut Ctx, _: Self::Form) -> Result<Data> {
		if cx.help.input.mode() == InputMode::Normal {
			return act!(help:close, cx);
		}

		act!(escape, cx.help.input)?;
		succ!(render!());
	}
}
