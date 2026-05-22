use anyhow::Result;
use yazi_macro::succ;
use yazi_parser::VoidForm;
use yazi_shared::data::Data;

use crate::{Actor, Ctx};

pub struct Stop;

impl Actor for Stop {
	type Form = VoidForm;

	const NAME: &str = "stop";

	fn act(cx: &mut Ctx, _: Self::Form) -> Result<Data> {
		cx.active_mut().preview.reset_image();

		*cx.term = None;

		succ!();
	}
}
