use anyhow::Result;
use yazi_macro::{act, render, succ};
use yazi_parser::input::CloseOpt;
use yazi_shared::{errors::InputError, event::Data};

use crate::{Actor, Ctx};

pub struct Close;

impl Actor for Close {
	type Options = CloseOpt;

	const NAME: &'static str = "close";

	fn act(cx: &mut Ctx, opt: Self::Options) -> Result<Data> {
		let input = &mut cx.input;
		input.visible = false;
		input.ticket.next();

		if let Some(tx) = input.tx.take() {
			let value = input.snap().value.clone();
			_ = tx.send(if opt.submit { Ok(value) } else { Err(InputError::Canceled(value)) });
		}

		act!(cmp:close, cx)?;
		succ!(render!());
	}
}
