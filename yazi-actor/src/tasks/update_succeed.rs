use anyhow::Result;
use yazi_macro::succ;
use yazi_parser::tasks::UpdateSucceedOpt;
use yazi_shared::event::Data;

use crate::{Actor, Ctx};

pub struct UpdateSucceed;

impl Actor for UpdateSucceed {
	type Options = UpdateSucceedOpt;

	const NAME: &str = "update_succeed";

	fn act(cx: &mut Ctx, opt: Self::Options) -> Result<Data> {
		cx.mgr.watcher.push_files(opt.urls);
		succ!();
	}
}
