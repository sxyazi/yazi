use anyhow::Result;
use yazi_core::mgr::Yanked;
use yazi_macro::{render, succ};
use yazi_parser::mgr::UpdateYankedOpt;
use yazi_shared::event::Data;

use crate::{Actor, Ctx};

pub struct UpdateYanked;

impl Actor for UpdateYanked {
	type Options = UpdateYankedOpt;

	const NAME: &'static str = "update_yanked";

	fn act(cx: &mut Ctx, opt: Self::Options) -> Result<Data> {
		if opt.urls.is_empty() && cx.mgr.yanked.is_empty() {
			succ!();
		}

		cx.mgr.yanked = Yanked::new(opt.cut, opt.urls);
		succ!(render!());
	}
}
