use anyhow::Result;
use yazi_macro::succ;
use yazi_parser::mgr::PeekForm;
use yazi_shared::data::Data;

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
		let folder = cx.tab().hovered_folder().map(|f| (f.offset, f.file.clone()));

		if !cx.tab().preview.same_url(&hovered.url) {
			cx.tab_mut().preview.skip = folder.as_ref().map(|f| f.0).unwrap_or_default();
		}
		if !cx.tab().preview.same_file(&hovered, &mime) {
			cx.tab_mut().preview.reset();
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

		if let Some((_, file)) = folder {
			cx.core.mgr.watcher.refresher.refresh([file]);
		} else if hovered.is_dir() {
			cx.core.mgr.watcher.refresher.load(&hovered);
		}

		cx.tab_mut().preview.go(hovered, mime, form.force);
		succ!();
	}
}
