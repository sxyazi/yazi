use anyhow::Result;
use yazi_core::tab::Folder;
use yazi_macro::{render, render_and, succ};
use yazi_parser::mgr::ExcludeAddForm;
use yazi_shared::data::Data;
use yazi_shim::OptionExt;

use crate::{Actor, Ctx, act};

pub struct ExcludeAdd;

impl Actor for ExcludeAdd {
	type Form = ExcludeAddForm;

	const NAME: &str = "exclude_add";

	fn act(cx: &mut Ctx, form: Self::Form) -> Result<Data> {
		if form.patterns.is_empty() {
			succ!();
		}

		let hovered = cx.hovered().map(|f| f.key()).owned();
		let apply = |f: &mut Folder| {
			if !f.set_exclude_patterns(form.patterns.clone()) {
				false
			} else if f.stage.is_loading() {
				render!();
				false
			} else {
				render_and!(f.entries.catchup_revision())
			}
		};

		// Apply to CWD and parent
		if apply(cx.current_mut()) | cx.parent_mut().is_some_and(apply) {
			act!(mgr:hover, cx)?;
			act!(mgr:update_paged, cx)?;
		}

		// Apply to hovered
		if let Some(h) = cx.hovered_folder_mut()
			&& apply(h)
		{
			render!(h.repos(None));
			act!(mgr:peek, cx, true)?;
		} else if cx.hovered().map(|f| f.key()) != hovered.as_ref().map(Into::into) {
			act!(mgr:peek, cx)?;
			act!(mgr:watch, cx).ok();
		}

		succ!()
	}
}
