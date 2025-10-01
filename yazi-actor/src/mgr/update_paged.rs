use anyhow::Result;
use yazi_macro::succ;
use yazi_parser::mgr::UpdatePagedOpt;
use yazi_shared::data::Data;

use crate::{Actor, Ctx};

pub struct UpdatePaged;

impl Actor for UpdatePaged {
	type Options = UpdatePagedOpt;

	const NAME: &str = "update_paged";

	fn act(cx: &mut Ctx, opt: Self::Options) -> Result<Data> {
		if opt.only_if.is_some_and(|u| u != *cx.cwd()) {
			succ!();
		}

		let targets = cx.current().paginate(opt.page.unwrap_or(cx.current().page));
		if !targets.is_empty() {
			cx.tasks().fetch_paged(targets, &cx.mgr.mimetype);
			cx.tasks().preload_paged(targets, &cx.mgr.mimetype);
		}
		succ!();
	}
}
