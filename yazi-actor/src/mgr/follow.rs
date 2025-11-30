use anyhow::Result;
use yazi_fs::path::clean_url;
use yazi_macro::{act, succ};
use yazi_parser::{VoidOpt, mgr::CdSource};
use yazi_shared::{data::Data, url::UrlLike};

use crate::{Actor, Ctx};

pub struct Follow;

impl Actor for Follow {
	type Options = VoidOpt;

	const NAME: &str = "follow";

	fn act(cx: &mut Ctx, _: Self::Options) -> Result<Data> {
		let Some(file) = cx.hovered() else { succ!() };
		let Some(link_to) = &file.link_to else { succ!() };
		let Some(parent) = file.url.parent() else { succ!() };
		let Ok(joined) = parent.try_join(link_to) else { succ!() };
		act!(mgr:reveal, cx, (clean_url(joined), CdSource::Follow))
	}
}
