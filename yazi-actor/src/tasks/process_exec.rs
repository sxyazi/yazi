use anyhow::Result;
use yazi_macro::succ;
use yazi_proxy::options::ProcessExecOpt;
use yazi_shared::event::Data;

use crate::{Actor, Ctx};

pub struct ProcessExec;

impl Actor for ProcessExec {
	type Options = ProcessExecOpt;

	const NAME: &'static str = "process_exec";

	fn act(cx: &mut Ctx, opt: Self::Options) -> Result<Data> {
		succ!(cx.tasks.scheduler.process_open(opt));
	}
}
