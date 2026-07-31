use anyhow::Result;
use yazi_macro::succ;
use yazi_parser::app::StopForm;
use yazi_shared::data::Data;

use crate::{Actor, Ctx};

pub struct Stop;

impl Actor for Stop {
	type Form = StopForm;

	const NAME: &str = "stop";

	fn act(cx: &mut Ctx, Self::Form { replier }: Self::Form) -> Result<Data> {
		cx.active_mut().preview.reset_image();

		*cx.term = None;

		if let Some(replier) = replier {
			replier.send(Ok(Data::Nil)).ok();
		}

		succ!();
	}
}
