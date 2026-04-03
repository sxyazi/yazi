use anyhow::Result;
use yazi_core::mgr::Yanked;
use yazi_macro::{act, render};
use yazi_parser::mgr::YankForm;
use yazi_shared::{data::Data, url::UrlBufCov};

use crate::{Actor, Ctx};

pub struct Yank;

impl Actor for Yank {
	type Form = YankForm;

	const NAME: &str = "yank";

	fn act(cx: &mut Ctx, form: Self::Form) -> Result<Data> {
		act!(mgr:escape_visual, cx)?;

		cx.mgr.yanked =
			Yanked::new(form.cut, cx.tab().selected_or_hovered().cloned().map(UrlBufCov).collect());
		render!(cx.mgr.yanked.catchup_revision(true));

		act!(mgr:escape_select, cx)
	}
}
