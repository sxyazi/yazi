use anyhow::Result;
use yazi_macro::{act, render, succ};
use yazi_parser::VoidOpt;
use yazi_shared::data::Data;

use crate::Actor;

pub struct Unyank;

impl Actor for Unyank {
	type Options = VoidOpt;

	const NAME: &str = "unyank";

	fn act(cx: &mut crate::Ctx, _: Self::Options) -> Result<Data> {
		let repeek = cx.hovered().is_some_and(|f| f.is_dir() && cx.mgr.yanked.contains_in(&f.url));
		cx.mgr.yanked.clear();

		render!(cx.mgr.yanked.catchup_revision(false));
		if repeek {
			act!(mgr:peek, cx, true)?;
		}

		succ!();
	}
}
