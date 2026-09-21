use anyhow::Result;
use yazi_macro::{succ, tab};
use yazi_parser::mgr::PeekForm;
use yazi_shared::{data::Data, url::UrlLike};

use crate::{Actor, Ctx};

pub struct Peek;

impl Actor for Peek {
	type Form = PeekForm;

	const NAME: &str = "peek";

	fn act(cx: &mut Ctx, form: Self::Form) -> Result<Data> {
		let Some(hovered) = cx.hovered().cloned() else {
			succ!(cx.tab_mut().preview.reset());
		};
		if cx.term.is_none() {
			succ!(cx.tab_mut().preview.reset_image());
		}

		let mime = cx.mgr.mimetype.owned(&hovered.url).unwrap_or_default();

		if !cx.tab().preview.same_url(&hovered.url) {
			cx.tab_mut().preview.skip = cx.hovered_folder().map(|f| f.offset).unwrap_or_default();
		}
		if !cx.tab().preview.same_file(&hovered, &mime) {
			cx.tab_mut().preview.reset();
		}
		if !cx.tab().preview.same_folder(&hovered.url) {
			cx.tab_mut().preview.folder_lock = None;
		}
		if matches!(form.only_if, Some(u) if u != hovered.url) {
			succ!();
		}

		if let Some(skip) = form.skip {
			let preview = &mut cx.tab_mut().preview;
			if form.upper_bound {
				preview.skip = preview.skip.min(skip);
			} else {
				preview.skip = skip;
			}
		}

		let unlocked = cx.tab().preview.folder_lock.is_none();
		if let Some(folder) = tab!(cx).hovered_folder_mut() {
			let op = folder.take_refresh();
			if unlocked || op.is_force() {
				cx.tab_mut().preview.folder_lock = Some(folder.to_url());
				cx.core.mgr.watcher.refresher.request([op]);
			}
		} else if hovered.is_dir() && unlocked {
			cx.tab_mut().preview.folder_lock = Some(hovered.to_url());
			cx.core.mgr.watcher.refresher.load(&hovered);
		}

		cx.tab_mut().preview.go(hovered, mime, form.force);
		succ!();
	}
}
