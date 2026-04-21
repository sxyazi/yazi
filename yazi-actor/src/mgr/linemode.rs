use anyhow::Result;
use yazi_macro::{render, succ};
use yazi_parser::mgr::LinemodeForm;
use yazi_shared::data::Data;

use crate::{Actor, Ctx};

pub struct Linemode;

impl Actor for Linemode {
	type Form = LinemodeForm;

	const NAME: &str = "linemode";

	fn act(cx: &mut Ctx, form: Self::Form) -> Result<Data> {
		let tab = cx.tab_mut();

		if form.new != *tab.pref.linemode {
			tab.pref.linemode = form.new.into();
			render!();
		}

		succ!();
	}
}
