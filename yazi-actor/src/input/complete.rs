use anyhow::Result;
use yazi_macro::{act, succ};
use yazi_parser::input::CompleteOpt;
use yazi_shared::data::Data;

use crate::{Actor, Ctx};

pub struct Complete;

impl Actor for Complete {
	type Options = CompleteOpt;

	const NAME: &str = "complete";

	fn act(cx: &mut Ctx, opt: Self::Options) -> Result<Data> {
		let input = &mut cx.input;
		if !input.visible || input.ticket.current() != opt.ticket {
			succ!();
		}

		act!(complete, input, opt)
	}
}
