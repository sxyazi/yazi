use anyhow::Result;
use yazi_macro::{act, render, succ};
use yazi_parser::input::CloseForm;
use yazi_shared::data::Data;
use yazi_widgets::input::InputEvent;

use crate::{Actor, Ctx};

pub struct Close;

impl Actor for Close {
	type Form = CloseForm;

	const NAME: &str = "close";

	fn act(cx: &mut Ctx, form: Self::Form) -> Result<Data> {
		let input = &mut cx.input;
		input.visible = false;
		input.ticket.next();

		if let Some(tx) = input.tx.take() {
			let value = input.snap().value.clone();
			_ = tx.send(if form.submit { InputEvent::Submit(value) } else { InputEvent::Cancel(value) });
		}

		act!(cmp:close, cx)?;
		succ!(render!());
	}
}
