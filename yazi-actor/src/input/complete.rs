use anyhow::Result;
use yazi_macro::{act, succ};
use yazi_shared::data::Data;
use yazi_widgets::input::parser::CompleteOpt;

use crate::{Actor, Ctx};

pub struct Complete;

impl Actor for Complete {
	type Form = CompleteOpt;

	const NAME: &str = "complete";

	fn act(cx: &mut Ctx, form: Self::Form) -> Result<Data> {
		let input = &mut cx.input;
		if !input.visible || input.ticket.current() != form.ticket {
			succ!();
		}

		act!(complete, input, form)
	}
}
