use anyhow::Result;
use yazi_macro::succ;
use yazi_parser::mgr::UpdateTasksOpt;
use yazi_shared::event::Data;

use crate::{Actor, Ctx};

pub struct UpdateTasks;

impl Actor for UpdateTasks {
	type Options = UpdateTasksOpt;

	const NAME: &'static str = "update_tasks";

	fn act(cx: &mut Ctx, opt: Self::Options) -> Result<Data> {
		cx.mgr.watcher.push_files(opt.urls);
		succ!();
	}
}
