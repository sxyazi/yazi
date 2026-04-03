use anyhow::Result;
use yazi_macro::{act, succ};
use yazi_parser::mgr::PasteForm;
use yazi_shared::data::Data;

use crate::{Actor, Ctx};

pub struct Paste;

impl Actor for Paste {
	type Form = PasteForm;

	const NAME: &str = "paste";

	fn act(cx: &mut Ctx, form: Self::Form) -> Result<Data> {
		let mgr = &mut cx.core.mgr;
		let tab = &mgr.tabs[cx.tab];

		let dest = tab.cwd();
		if mgr.yanked.cut {
			cx.core.tasks.file_cut(&mgr.yanked, dest, form.force);

			mgr.tabs.iter_mut().for_each(|t| _ = t.selected.remove_many(&*mgr.yanked));
			act!(mgr:unyank, cx)
		} else {
			succ!(cx.core.tasks.file_copy(&mgr.yanked, dest, form.force, form.follow));
		}
	}
}
