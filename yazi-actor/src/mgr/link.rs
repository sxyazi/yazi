use anyhow::Result;
use yazi_macro::succ;
use yazi_parser::mgr::LinkForm;
use yazi_shared::data::Data;

use crate::{Actor, Ctx};

pub struct Link;

impl Actor for Link {
	type Form = LinkForm;

	const NAME: &str = "link";

	fn act(cx: &mut Ctx, form: Self::Form) -> Result<Data> {
		let mgr = &mut cx.core.mgr;
		let tab = &mgr.tabs[cx.tab];

		if !mgr.yanked.cut {
			cx.core.tasks.file_link(&mgr.yanked, tab.cwd(), form.relative, form.force);
		}

		succ!();
	}
}
