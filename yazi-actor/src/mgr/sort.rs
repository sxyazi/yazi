use anyhow::Result;
use yazi_core::tab::Folder;
use yazi_dds::spark::SparkKind;
use yazi_fs::{FilesSorter, FolderStage};
use yazi_macro::{act, render, render_and, succ};
use yazi_parser::mgr::SortOpt;
use yazi_shared::{Source, data::Data};

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
		let hovered = cx.hovered().map(|f| f.urn().to_owned());
		let apply = |f: &mut Folder| {
			if f.stage == FolderStage::Loading {
				render!();
				false
			} else {
				f.files.set_sorter(sorter);
				render_and!(f.files.catchup_revision())
			}
		};

		// Apply to CWD and parent
		if let (a, Some(b)) = (apply(cx.current_mut()), cx.parent_mut().map(apply))
			&& (a | b)
		{
			act!(mgr:hover, cx)?;
			act!(mgr:update_paged, cx)?;
			cx.tasks.prework_sorted(&cx.mgr.tabs[cx.tab].current.files);
		}

		// Apply to hovered
		if let Some(h) = cx.hovered_folder_mut()
			&& apply(h)
		{
			render!(h.repos(None));
			act!(mgr:peek, cx, true)?;
		} else if cx.hovered().map(|f| f.urn()) != hovered.as_ref().map(Into::into) {
			act!(mgr:peek, cx)?;
			act!(mgr:watch, cx)?;
		}

		succ!();
	}

	fn hook(cx: &Ctx, _: &Self::Options) -> Option<SparkKind> {
		match cx.source() {
			Source::Ind => Some(SparkKind::IndSort),
			Source::Key => Some(SparkKind::KeySort),
			_ => None,
		}
	}
}
