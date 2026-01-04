use anyhow::Result;
use yazi_macro::{act, succ};
use yazi_parser::{VoidOpt, mgr::CdSource};
use yazi_shared::{data::Data, url::UrlLike};

use crate::{Actor, Ctx};

pub struct Leave;

impl Actor for Leave {
	type Options = VoidOpt;

	const NAME: &str = "leave";

	fn act(cx: &mut Ctx, _: Self::Options) -> Result<Data> {
		let url = cx
			.hovered()
			.and_then(|h| h.url.parent())
			.filter(|u| u != cx.cwd())
			.or_else(|| cx.cwd().parent());

		let Some(mut url) = url else { succ!() };
		if url.is_search() {
			url = url.as_regular()?;
		}

		act!(mgr:cd, cx, (url, CdSource::Leave))
	}
}
