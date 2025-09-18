use anyhow::Result;
use yazi_config::YAZI;
use yazi_macro::succ;
use yazi_parser::mgr::PeekOpt;
use yazi_proxy::HIDER;
use yazi_shared::event::Data;

use crate::{Actor, Ctx};

pub struct Peek;

impl Actor for Peek {
	type Options = PeekOpt;

	const NAME: &str = "peek";

	fn act(cx: &mut Ctx, opt: Self::Options) -> Result<Data> {
		let Some(hovered) = cx.hovered().cloned() else {
			succ!(cx.tab_mut().preview.reset());
		};
		if HIDER.try_acquire().is_err() {
			succ!(cx.tab_mut().preview.reset_image());
		}

		let mime = cx.mgr.mimetype.by_file_owned(&hovered).unwrap_or_default();
		let folder = cx.tab().hovered_folder().map(|f| (f.offset, f.cha));

		let same_url = cx.tab().preview.same_url(&hovered.url);
		let same_file = cx.tab().preview.same_file(&hovered, &mime);

		{
			let preview = &mut cx.tab_mut().preview;
			if !same_url {
				preview.skip = folder.map(|f| f.0).unwrap_or_default();
				preview.run = 0;
			}
			if !same_file {
				preview.reset();
			}

			if let Some(step) = opt.cycle {
				if step == 0 {
					preview.run = 0;
				} else if let Some(previewer) = YAZI.plugin.previewer(&hovered.url, &mime) {
					let len = previewer.len();
					if len == 0 {
						preview.run = 0;
					} else {
						let len_isize = len as isize;
						let current = (preview.run % len) as isize;
						let next = (current + step as isize).rem_euclid(len_isize);
						preview.run = next as usize;
					}
				} else {
					preview.run = 0;
				}
			}

			if let Some(skip) = opt.skip {
				if opt.upper_bound {
					preview.skip = preview.skip.min(skip);
				} else {
					preview.skip = skip;
				}
			}
		}

		if matches!(opt.only_if, Some(u) if u != hovered.url) {
			succ!();
		}

		let force = opt.force || opt.cycle.is_some();
		if hovered.is_dir() {
			cx.tab_mut().preview.go_folder(hovered, folder.map(|(_, cha)| cha), force);
		} else {
			cx.tab_mut().preview.go(hovered, mime, force);
		}
		succ!();
	}
}
