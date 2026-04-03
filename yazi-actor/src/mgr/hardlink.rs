use anyhow::Result;
use yazi_macro::succ;
use yazi_parser::mgr::HardlinkForm;
use yazi_shared::data::Data;

use crate::{Actor, Ctx};

pub struct Hardlink;

impl Actor for Hardlink {
	type Form = HardlinkForm;

	const NAME: &str = "hardlink";

	fn act(cx: &mut Ctx, form: Self::Form) -> Result<Data> {
		let mgr = &mut cx.core.mgr;
		let tab = &mgr.tabs[cx.tab];

		if !mgr.yanked.cut {
			cx.core.tasks.file_hardlink(&mgr.yanked, tab.cwd(), form.force, form.follow);
		}

		succ!();
	}
}
