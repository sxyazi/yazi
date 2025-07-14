use anyhow::Result;
use yazi_macro::succ;
use yazi_parser::which::CallbackOpt;
use yazi_shared::event::Data;

use crate::{Actor, Ctx};

pub struct Callback;

impl Actor for Callback {
	type Options = CallbackOpt;

	const NAME: &'static str = "callback";

	fn act(_: &mut Ctx, opt: Self::Options) -> Result<Data> {
		opt.tx.try_send(opt.idx)?;
		succ!();
	}
}
