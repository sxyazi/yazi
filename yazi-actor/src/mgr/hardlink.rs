use anyhow::Result;
use yazi_macro::succ;
use yazi_parser::mgr::HardlinkOpt;
use yazi_shared::data::Data;

use crate::{Actor, Ctx};

pub struct Hardlink;

impl Actor for Hardlink {
	type Options = HardlinkOpt;

	const NAME: &str = "hardlink";

	fn act(cx: &mut Ctx, opt: Self::Options) -> Result<Data> {
		let mgr = &mut cx.core.mgr;
		let tab = &mgr.tabs[cx.tab];

		if !mgr.yanked.cut {
			cx.core.tasks.file_hardlink(&mgr.yanked, tab.cwd(), opt.force, opt.follow);
		}

		succ!();
	}
}
