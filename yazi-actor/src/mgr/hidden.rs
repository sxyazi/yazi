use anyhow::Result;
use yazi_core::tab::Folder;
use yazi_fs::FolderStage;
use yazi_macro::{act, render, render_and, succ};
use yazi_parser::{mgr::HiddenForm, spark::SparkKind};
use yazi_shared::{Source, data::Data};
use yazi_shim::OptionExt;

use crate::{Actor, Ctx};

pub struct Hidden;

impl Actor for Hidden {
	type Form = HiddenForm;

	const NAME: &str = "hidden";

	fn act(cx: &mut Ctx, form: Self::Form) -> Result<Data> {
		let state = form.state.bool(cx.tab().pref.show_hidden);
		cx.tab_mut().pref.show_hidden = state;

		let hovered = cx.hovered().map(|f| f.entry_key()).owned();
		let apply = |f: &mut Folder| {
			if f.stage == FolderStage::Loading {
				render!();
				false
			} else {
				f.entries.set_show_hidden(state);
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
		} else if cx.hovered().map(|f| f.entry_key()) != hovered.as_ref().map(Into::into) {
			act!(mgr:peek, cx)?;
			act!(mgr:watch, cx)?;
		}

		succ!()
	}

	fn hook(cx: &Ctx, _: &Self::Form) -> Option<SparkKind> {
		match cx.source() {
			Source::Ind => Some(SparkKind::IndHidden),
			Source::Key => Some(SparkKind::KeyHidden),
			_ => None,
		}
	}
}
