use anyhow::Result;
use yazi_config::YAZI;
use yazi_macro::succ;
use yazi_parser::mgr::SeekOpt;
use yazi_plugin::isolate;
use yazi_shared::event::Data;

use crate::{Actor, Ctx};

pub struct Seek;

impl Actor for Seek {
	type Options = SeekOpt;

	const NAME: &'static str = "seek";

	fn act(cx: &mut Ctx, opt: Self::Options) -> Result<Data> {
		let Some(hovered) = cx.hovered() else {
			succ!(cx.tab_mut().preview.reset());
		};

		let Some(mime) = cx.mgr.mimetype.by_file(hovered) else {
			succ!(cx.tab_mut().preview.reset());
		};

		let Some(previewer) = YAZI.plugin.previewer(&hovered.url, mime) else {
			succ!(cx.tab_mut().preview.reset());
		};

		isolate::seek_sync(&previewer.run, hovered.clone(), opt.units);
		succ!();
	}
}
