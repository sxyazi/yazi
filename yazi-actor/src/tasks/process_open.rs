use anyhow::Result;
use yazi_macro::succ;
use yazi_parser::tasks::ProcessOpenOpt;
use yazi_shared::data::Data;

use crate::{Actor, Ctx};

pub struct ProcessOpen;

impl Actor for ProcessOpen {
	type Options = ProcessOpenOpt;

	const NAME: &str = "process_open";

	fn act(cx: &mut Ctx, opt: Self::Options) -> Result<Data> {
		succ!(cx.tasks.scheduler.process_open(opt));
	}
}
