use std::ops::DerefMut;

use anyhow::Result;
use yazi_macro::{act, render, succ};
use yazi_shared::data::Data;
use yazi_widgets::input::InputOpt;

use crate::{Actor, Ctx};

pub struct Show;

impl Actor for Show {
	type Options = InputOpt;

	const NAME: &str = "show";

	fn act(cx: &mut Ctx, opt: Self::Options) -> Result<Data> {
		act!(input:close, cx)?;

		let input = &mut cx.input;
		input.visible = true;
		input.title = opt.cfg.title.clone();
		input.position = opt.cfg.position;
		*input.deref_mut() = yazi_widgets::input::Input::new(opt)?;

		succ!(render!());
	}
}
