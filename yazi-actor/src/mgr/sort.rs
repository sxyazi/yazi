use anyhow::Result;
use yazi_core::tab::Folder;
use yazi_fs::{FilesSorter, FolderStage};
use yazi_macro::{act, render, succ};
use yazi_parser::mgr::SortOpt;
use yazi_shared::event::Data;

use crate::{Actor, Ctx};

pub struct Sort;

impl Actor for Sort {
	type Options = SortOpt;

	const NAME: &str = "sort";

	fn act(cx: &mut Ctx, opt: Self::Options) -> Result<Data> {
		let pref = &mut cx.tab_mut().pref;
		pref.sort_by = opt.by.unwrap_or(pref.sort_by);
		pref.sort_reverse = opt.reverse.unwrap_or(pref.sort_reverse);
		pref.sort_dir_first = opt.dir_first.unwrap_or(pref.sort_dir_first);
		pref.sort_sensitive = opt.sensitive.unwrap_or(pref.sort_sensitive);
		pref.sort_translit = opt.translit.unwrap_or(pref.sort_translit);

		let sorter = FilesSorter::from(&*pref);
		let hovered = cx.hovered().map(|f| f.url_owned());
		let apply = |f: &mut Folder| {
			if f.stage == FolderStage::Loading {
				render!();
			} else {
				f.files.set_sorter(sorter);
				render!(f.files.catchup_revision());
			}
		};

		// Apply to CWD and parent
		apply(cx.current_mut());
		cx.parent_mut().map(apply);

		// Repos CWD and parent
		act!(mgr:hover, cx)?;

		// Apply to and repos hovered folder
		if let Some(h) = cx.hovered_folder_mut() {
			apply(h);
			render!(h.repos(None));
		}

		if hovered.as_ref() != cx.hovered().map(|f| &f.url) {
			act!(mgr:peek, cx)?;
			act!(mgr:watch, cx)?;
		} else if cx.hovered().is_some_and(|f| f.is_dir()) {
			act!(mgr:peek, cx, true)?;
		}

		act!(mgr:update_paged, cx)?;
		succ!(cx.tasks.prework_sorted(&cx.mgr.tabs[cx.tab].current.files));
	}
}
