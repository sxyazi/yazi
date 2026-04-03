use anyhow::Result;
use yazi_macro::succ;
use yazi_parser::app::StopForm;
use yazi_shared::data::Data;

use crate::{Actor, Ctx};

pub struct Stop;

impl Actor for Stop {
	type Form = StopForm;

	const NAME: &str = "stop";

	fn act(cx: &mut Ctx, form: Self::Form) -> Result<Data> {
		cx.active_mut().preview.reset_image();

		// We need to destroy the `term` first before stopping the `signals`
		// to prevent any signal from triggering the term to render again
		// while the app is being suspended.
		*cx.term = None;

		form.tx.send((false, form.replier))?;

		succ!();
	}
}
