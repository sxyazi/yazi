use anyhow::Result;
use yazi_core::tab::Folder;
use yazi_fs::{FilesSorter, FolderStage};
use yazi_macro::{act, render, render_and, succ};
use yazi_parser::{mgr::SortForm, spark::SparkKind};
use yazi_shared::{Source, data::Data};

use crate::{Actor, Ctx};

pub struct Sort;

impl Actor for Sort {
	type Form = SortForm;

	const NAME: &str = "sort";

	fn act(cx: &mut Ctx, form: Self::Form) -> Result<Data> {
		let pref = &mut cx.tab_mut().pref;
		pref.sort_by = form.by.unwrap_or(pref.sort_by);
		pref.sort_reverse = form.reverse.unwrap_or(pref.sort_reverse);
		pref.sort_dir_first = form.dir_first.unwrap_or(pref.sort_dir_first);
		pref.sort_sensitive = form.sensitive.unwrap_or(pref.sort_sensitive);
		pref.sort_translit = form.translit.unwrap_or(pref.sort_translit);
		pref.sort_fallback = form.fallback.unwrap_or(pref.sort_fallback);

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

	fn hook(cx: &Ctx, _: &Self::Form) -> Option<SparkKind> {
		match cx.source() {
			Source::Ind => Some(SparkKind::IndSort),
			Source::Key => Some(SparkKind::KeySort),
			_ => None,
		}
	}
}
