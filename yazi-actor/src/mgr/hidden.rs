use anyhow::Result;
use yazi_macro::{act, succ};
use yazi_parser::mgr::HiddenOpt;
use yazi_shared::event::Data;

use crate::{Actor, Ctx};

pub struct Hidden;

impl Actor for Hidden {
	type Options = HiddenOpt;

	const NAME: &'static str = "hidden";

	fn act(cx: &mut Ctx, opt: Self::Options) -> Result<Data> {
		let tab = cx.tab_mut();
		tab.pref.show_hidden = opt.state.unwrap_or(!tab.pref.show_hidden);

		let hovered = tab.hovered().map(|f| f.url_owned());
		tab.apply_files_attrs();

		if hovered.as_ref() != tab.hovered().map(|f| &f.url) {
			act!(mgr:hover, cx, hovered)?;
			act!(mgr:peek, cx)?;
			act!(mgr:watch, cx)?;
		} else if tab.hovered().is_some_and(|f| f.is_dir()) {
			act!(mgr:peek, cx, true)?;
		}

		act!(mgr:update_paged, cx)?;
		succ!();
	}
}
