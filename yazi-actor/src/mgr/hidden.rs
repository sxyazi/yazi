use anyhow::Result;
use yazi_core::tab::Folder;
use yazi_fs::FolderStage;
use yazi_macro::{act, render, render_and, succ};
use yazi_parser::mgr::HiddenOpt;
use yazi_shared::data::Data;

use crate::{Actor, Ctx};

pub struct Hidden;

impl Actor for Hidden {
	type Options = HiddenOpt;

	const NAME: &str = "hidden";

	fn act(cx: &mut Ctx, opt: Self::Options) -> Result<Data> {
		let state = opt.state.bool(cx.tab().pref.show_hidden);
		cx.tab_mut().pref.show_hidden = state;

		let hovered = cx.hovered().map(|f| f.urn().to_owned());
		let apply = |f: &mut Folder| {
			if f.stage == FolderStage::Loading {
				render!();
				false
			} else {
				f.files.set_show_hidden(state);
				render_and!(f.files.catchup_revision())
			}
		};

		// Apply to CWD and parent
		if let (a, Some(b)) = (apply(cx.current_mut()), cx.parent_mut().map(apply))
			&& (a | b)
		{
			act!(mgr:hover, cx)?;
			act!(mgr:update_paged, cx)?;
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

		succ!()
	}
}
