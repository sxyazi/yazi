use anyhow::Result;
use yazi_macro::{act, succ};
use yazi_parser::mgr::SortOpt;
use yazi_shared::event::Data;

use crate::{Actor, Ctx};

pub struct Sort;

impl Actor for Sort {
	type Options = SortOpt;

	const NAME: &str = "sort";

	fn act(cx: &mut Ctx, opt: Self::Options) -> Result<Data> {
		let mut new = cx.tab().pref.clone();
		new.sort_by = opt.by.unwrap_or(new.sort_by);
		new.sort_reverse = opt.reverse.unwrap_or(new.sort_reverse);
		new.sort_dir_first = opt.dir_first.unwrap_or(new.sort_dir_first);
		new.sort_sensitive = opt.sensitive.unwrap_or(new.sort_sensitive);
		new.sort_translit = opt.translit.unwrap_or(new.sort_translit);

		if new == cx.tab().pref {
			succ!();
		}

		cx.tab_mut().pref = new;
		cx.tab_mut().apply_files_attrs();
		act!(mgr:hover, cx)?;

		cx.tasks.prework_sorted(&cx.mgr.tabs[cx.tab].current.files);
		act!(mgr:peek, cx)?;
		act!(mgr:watch, cx)?;
		act!(mgr:update_paged, cx)?;

		succ!();
	}
}
