use anyhow::Result;
use yazi_fs::clean_url;
use yazi_macro::{act, succ};
use yazi_parser::VoidOpt;
use yazi_shared::event::Data;

use crate::{Actor, Ctx};

pub struct Follow;

impl Actor for Follow {
	type Options = VoidOpt;

	const NAME: &'static str = "follow";

	fn act(cx: &mut Ctx, _: Self::Options) -> Result<Data> {
		let Some(file) = cx.hovered() else { succ!() };
		let Some(link_to) = &file.link_to else { succ!() };

		if link_to.is_absolute() {
			act!(mgr:reveal, cx, link_to.to_owned())
		} else if let Some(p) = file.url.parent_url() {
			act!(mgr:reveal, cx, clean_url(&p.join(link_to)))
		} else {
			succ!()
		}
	}
}
