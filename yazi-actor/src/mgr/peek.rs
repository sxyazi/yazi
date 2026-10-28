use anyhow::Result;
use yazi_core::Ctx;
use yazi_macro::succ;
use yazi_parser::mgr::PeekForm;
use yazi_shared::{data::Data, url::UrlLike};

use crate::Actor;

pub struct Peek;

impl Actor for Peek {
	type Form = PeekForm;

	const NAME: &str = "peek";

	fn act(cx: &mut Ctx, form: Self::Form) -> Result<Data> {
		let Some(hovered) = cx.hovered().cloned() else {
			succ!(cx.preview_mut().reset());
		};
		if cx.term.is_none() {
			succ!(cx.preview_mut().reset_image());
		} else if cx.scope.is_cancelled() {
			succ!();
		}

		let mime = cx.mgr.mimetype.owned(&hovered.url).unwrap_or_default();
		if !cx.preview().same_url(&hovered.url) {
			cx.preview_mut().skip = cx.hovered_folder().map(|f| f.offset).unwrap_or_default();
		}
		if !cx.preview().same_file(&hovered, &mime) {
			cx.preview_mut().reset();
		}
		if !cx.preview().same_folder(&hovered.url) {
			cx.preview_mut().folder_lock = None;
		}
		if matches!(form.only_if, Some(u) if u != hovered.url) {
			succ!();
		}

		if let Some(skip) = form.skip {
			let preview = &mut cx.preview_mut();
			if form.upper_bound {
				preview.skip = preview.skip.min(skip);
			} else {
				preview.skip = skip;
			}
		}

		let unlocked = cx.preview().folder_lock.is_none();
		if let Some(folder) = cx.hovered_folder_mut() {
			let op = folder.take_refresh();
			if unlocked || op.is_force() {
				cx.preview_mut().folder_lock = Some(folder.to_url());
				cx.core.mgr.watcher.refresher.request([op]);
			}
		} else if hovered.is_dir() && unlocked {
			cx.preview_mut().folder_lock = Some(hovered.to_url());
			cx.core.mgr.watcher.refresher.load(&hovered);
		}

		cx.preview_mut().go(hovered, mime, form.force);

		succ!();
	}
}
