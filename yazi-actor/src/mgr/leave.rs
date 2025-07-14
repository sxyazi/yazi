use anyhow::Result;
use yazi_macro::{act, succ};
use yazi_parser::{VoidOpt, tab::CdSource};
use yazi_shared::event::Data;

use crate::{Actor, Ctx};

pub struct Leave;

impl Actor for Leave {
	type Options = VoidOpt;

	const NAME: &'static str = "leave";

	fn act(cx: &mut Ctx, _: Self::Options) -> Result<Data> {
		let url = cx
			.hovered()
			.and_then(|h| h.url.parent_url())
			.filter(|u| u != cx.cwd())
			.or_else(|| cx.cwd().parent_url());

		if let Some(u) = url { act!(mgr:cd, cx, (u.into_regular(), CdSource::Leave)) } else { succ!() }
	}
}
