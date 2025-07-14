use anyhow::Result;
use yazi_fs::Filter;
use yazi_macro::{act, render, succ};
use yazi_parser::tab::FilterOpt;
use yazi_shared::event::Data;

use crate::{Actor, Ctx};

pub struct FilterDo;

impl Actor for FilterDo {
	type Options = FilterOpt;

	const NAME: &'static str = "filter_do";

	fn act(cx: &mut Ctx, opt: Self::Options) -> Result<Data> {
		let filter = if opt.query.is_empty() { None } else { Some(Filter::new(&opt.query, opt.case)?) };

		let hovered = cx.hovered().map(|f| f.urn_owned());
		if cx.current_mut().files.set_filter(filter) {
			cx.current_mut().repos(hovered.as_ref());
		}

		if cx.hovered().map(|f| f.urn()) != hovered.as_ref().map(|u| u.as_urn()) {
			act!(mgr:hover, cx)?;
			act!(mgr:peek, cx)?;
			act!(mgr:watch, cx)?;
		}

		if opt.done {
			act!(mgr:update_paged, cx)?;
		}

		succ!(render!());
	}
}
