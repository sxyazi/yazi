use anyhow::Result;
use yazi_macro::{render, succ};
use yazi_parser::mgr::LinemodeOpt;
use yazi_shared::data::Data;

use crate::{Actor, Ctx};

pub struct Linemode;

impl Actor for Linemode {
	type Options = LinemodeOpt;

	const NAME: &str = "linemode";

	fn act(cx: &mut Ctx, opt: Self::Options) -> Result<Data> {
		let tab = cx.tab_mut();

		if opt.new != tab.pref.linemode {
			tab.pref.linemode = opt.new.into_owned();
			render!();
		}

		succ!();
	}
}
