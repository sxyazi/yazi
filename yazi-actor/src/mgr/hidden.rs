use anyhow::Result;
use yazi_core::tab::Folder;
use yazi_fs::FolderStage;
use yazi_macro::{act, render};
use yazi_parser::mgr::HiddenOpt;
use yazi_shared::event::Data;

use crate::{Actor, Ctx};

pub struct Hidden;

impl Actor for Hidden {
	type Options = HiddenOpt;

	const NAME: &str = "hidden";

	fn act(cx: &mut Ctx, opt: Self::Options) -> Result<Data> {
		let state = opt.state.bool(cx.tab().pref.show_hidden);
		cx.tab_mut().pref.show_hidden = state;

		let hovered = cx.hovered().map(|f| f.url_owned());
		let apply = |f: &mut Folder| {
			if f.stage == FolderStage::Loading {
				render!();
			} else {
				f.files.set_show_hidden(state);
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

		act!(mgr:update_paged, cx)
	}
}
