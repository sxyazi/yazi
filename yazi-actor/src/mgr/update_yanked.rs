use anyhow::Result;
use yazi_core::mgr::Yanked;
use yazi_macro::{render, succ};
use yazi_parser::mgr::UpdateYankedForm;
use yazi_shared::data::Data;

use crate::{Actor, Ctx};

pub struct UpdateYanked;

impl Actor for UpdateYanked {
	type Form = UpdateYankedForm<'static>;

	const NAME: &str = "update_yanked";

	fn act(cx: &mut Ctx, form: Self::Form) -> Result<Data> {
		if form.urls.is_empty() && cx.mgr.yanked.is_empty() {
			succ!();
		}

		cx.mgr.yanked = Yanked::new(form.cut, form.0.urls.into_owned());
		succ!(render!());
	}
}
