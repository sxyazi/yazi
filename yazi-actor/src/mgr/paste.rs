use anyhow::Result;
use yazi_macro::{act, succ};
use yazi_parser::mgr::PasteOpt;
use yazi_shared::data::Data;

use crate::{Actor, Ctx};

pub struct Paste;

impl Actor for Paste {
	type Options = PasteOpt;

	const NAME: &str = "paste";

	fn act(cx: &mut Ctx, opt: Self::Options) -> Result<Data> {
		let mgr = &mut cx.core.mgr;
		let tab = &mgr.tabs[cx.tab];

		let dest = tab.cwd();
		if mgr.yanked.cut {
			cx.core.tasks.file_cut(&mgr.yanked, dest, opt.force);

			mgr.tabs.iter_mut().for_each(|t| _ = t.selected.remove_many(mgr.yanked.iter()));
			act!(mgr:unyank, cx)
		} else {
			succ!(cx.core.tasks.file_copy(&mgr.yanked, dest, opt.force, opt.follow));
		}
	}
}
