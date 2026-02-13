use anyhow::Result;
use yazi_macro::succ;
use yazi_parser::app::StopOpt;
use yazi_shared::data::Data;

use crate::{Actor, Ctx};

pub struct Stop;

impl Actor for Stop {
	type Options = StopOpt;

	const NAME: &str = "stop";

	fn act(cx: &mut Ctx, opt: Self::Options) -> Result<Data> {
		cx.active_mut().preview.reset_image();

		// We need to destroy the `term` first before stopping the `signals`
		// to prevent any signal from triggering the term to render again
		// while the app is being suspended.
		*cx.term = None;

		opt.tx.send((false, opt.token))?;

		succ!();
	}
}
