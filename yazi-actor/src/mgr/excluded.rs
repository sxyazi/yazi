use anyhow::Result;
use yazi_core::tab::Folder;
use yazi_fs::FolderStage;
use yazi_macro::{act, render, render_and, succ};
use yazi_parser::mgr::ExcludedOpt;
use yazi_shared::data::Data;

use crate::{Actor, Ctx};

pub struct Excluded;

impl Actor for Excluded {
	type Options = ExcludedOpt;

	const NAME: &str = "excluded";

	fn act(cx: &mut Ctx, opt: Self::Options) -> Result<Data> {
		let current_state = cx.tab().current.files.show_excluded();
		let state = opt.state.bool(current_state);

		let hovered = cx.hovered().map(|f| f.urn().to_owned());
		let apply = |f: &mut Folder| {
			if f.stage == FolderStage::Loading {
				render!();
				false
			} else {
				f.files.set_show_excluded(state);
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
