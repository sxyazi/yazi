use anyhow::Result;
use yazi_core::tab::Finder;
use yazi_macro::{act, render, succ};
use yazi_parser::mgr::FindDoOpt;
use yazi_shared::data::Data;

use crate::{Actor, Ctx};

pub struct FindDo;

impl Actor for FindDo {
	type Options = FindDoOpt;

	const NAME: &str = "find_do";

	fn act(cx: &mut Ctx, opt: Self::Options) -> Result<Data> {
		if opt.query.is_empty() {
			return act!(mgr:escape_find, cx);
		}

		let finder = Finder::new(&opt.query, opt.case)?;
		if matches!(&cx.tab().finder, Some(f) if f.filter == finder.filter) {
			succ!();
		}

		let step = if opt.prev {
			finder.prev(&cx.current().files, cx.current().cursor, true)
		} else {
			finder.next(&cx.current().files, cx.current().cursor, true)
		};

		if let Some(step) = step {
			act!(mgr:arrow, cx, step)?;
		}

		cx.tab_mut().finder = Some(finder);
		succ!(render!());
	}
}
