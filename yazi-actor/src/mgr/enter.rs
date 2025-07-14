use anyhow::Result;
use yazi_macro::{act, succ};
use yazi_parser::{VoidOpt, tab::CdSource};
use yazi_shared::event::Data;

use crate::{Actor, Ctx};

pub struct Enter;

impl Actor for Enter {
	type Options = VoidOpt;

	const NAME: &'static str = "enter";

	fn act(cx: &mut Ctx, _: Self::Options) -> Result<Data> {
		let Some(h) = cx.hovered().filter(|h| h.is_dir()) else { succ!() };

		act!(mgr:cd, cx, (h.url.to_regular(), CdSource::Enter))
	}
}
