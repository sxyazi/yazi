use anyhow::Result;
use yazi_macro::succ;
use yazi_parser::tasks::UpdateSucceedForm;
use yazi_shared::data::Data;

use crate::{Actor, Ctx};

pub struct UpdateSucceed;

impl Actor for UpdateSucceed {
	type Form = UpdateSucceedForm;

	const NAME: &str = "update_succeed";

	fn act(cx: &mut Ctx, opt: Self::Form) -> Result<Data> {
		if opt.urls.is_empty() {
			succ!();
		}

		// FIXME: todo!();
		// if opt.track
		// 	&& opt.batch == cx.tasks.scheduler.batch.current()
		// 	&& let Some((parent, urn)) = opt.urls[0].pair()
		// 	&& parent == *cx.cwd()
		// {
		// 	cx.tasks.scheduler.batch.next();
		// 	cx.current_mut().trace = Some(urn.into());
		// 	act!(mgr:hover, cx)?;
		// }

		cx.mgr.watcher.report(opt.urls);
		succ!();
	}
}
