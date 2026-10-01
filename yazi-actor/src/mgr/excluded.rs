use anyhow::Result;
use yazi_core::tab::Folder;
use yazi_macro::{render, render_and, succ};
use yazi_parser::mgr::ExcludedForm;
use yazi_shared::data::Data;
use yazi_shim::OptionExt;

use crate::{Actor, Ctx, act};

pub struct Excluded;

impl Actor for Excluded {
	type Form = ExcludedForm;

	const NAME: &str = "excluded";

	fn act(cx: &mut Ctx, form: Self::Form) -> Result<Data> {
		let state = form.state.bool(cx.tab().pref.show_excluded);
		cx.tab_mut().pref.show_excluded = state;

		let hovered = cx.hovered().map(|f| f.key()).owned();
		let apply = |f: &mut Folder| {
			if f.stage.is_loading() {
				render!();
				false
			} else {
				f.entries.set_show_excluded(state);
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
