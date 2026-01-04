use anyhow::Result;
use yazi_fs::Filter;
use yazi_macro::{act, render, succ};
use yazi_parser::mgr::FilterOpt;
use yazi_shared::data::Data;

use crate::{Actor, Ctx};

pub struct FilterDo;

impl Actor for FilterDo {
	type Options = FilterOpt;

	const NAME: &str = "filter_do";

	fn act(cx: &mut Ctx, opt: Self::Options) -> Result<Data> {
		let filter = if opt.query.is_empty() { None } else { Some(Filter::new(&opt.query, opt.case)?) };

		let hovered = cx.hovered().map(|f| f.urn().into());
		cx.current_mut().files.set_filter(filter);

		if cx.hovered().map(|f| f.urn()) != hovered.as_ref().map(Into::into) {
			act!(mgr:hover, cx, hovered)?;
			act!(mgr:peek, cx)?;
			act!(mgr:watch, cx)?;
		}

		if opt.done {
			act!(mgr:update_paged, cx)?;
		}

		succ!(render!());
	}
}
