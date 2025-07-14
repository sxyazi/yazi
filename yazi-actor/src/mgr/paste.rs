use anyhow::Result;
use yazi_macro::{act, succ};
use yazi_parser::mgr::PasteOpt;
use yazi_shared::event::Data;

use crate::{Actor, Ctx};

pub struct Paste;

impl Actor for Paste {
	type Options = PasteOpt;

	const NAME: &'static str = "paste";

	fn act(cx: &mut Ctx, opt: Self::Options) -> Result<Data> {
		let mgr = &mut cx.core.mgr;
		let tab = &mgr.tabs[cx.tab];
		let (src, dest) = (mgr.yanked.iter().collect::<Vec<_>>(), tab.cwd());

		if mgr.yanked.cut {
			cx.core.tasks.file_cut(&src, dest, opt.force);

			mgr.tabs.iter_mut().for_each(|t| _ = t.selected.remove_many(&src));
			act!(mgr:unyank, cx)
		} else {
			succ!(cx.core.tasks.file_copy(&src, dest, opt.force, opt.follow));
		}
	}
}
