use std::str::FromStr;

use anyhow::Result;
use yazi_fs::SortBy;
use yazi_macro::{act, succ};
use yazi_shared::event::{CmdCow, Data};

use crate::{Actor, Ctx};

pub struct Sort;

impl Actor for Sort {
	type Options = CmdCow;

	const NAME: &'static str = "sort";

	fn act(cx: &mut Ctx, c: Self::Options) -> Result<Data> {
		let mut new = cx.tab().pref.clone();
		new.sort_by = c.first_str().and_then(|s| SortBy::from_str(s).ok()).unwrap_or(new.sort_by);
		new.sort_reverse = c.maybe_bool("reverse").unwrap_or(new.sort_reverse);
		new.sort_dir_first = c.maybe_bool("dir-first").unwrap_or(new.sort_dir_first);
		new.sort_sensitive = c.maybe_bool("sensitive").unwrap_or(new.sort_sensitive);
		new.sort_translit = c.maybe_bool("translit").unwrap_or(new.sort_translit);

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
