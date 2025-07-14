use anyhow::Result;
use yazi_macro::{act, render, succ};
use yazi_parser::VoidOpt;
use yazi_shared::event::Data;

use crate::Actor;

pub struct Unyank;

impl Actor for Unyank {
	type Options = VoidOpt;

	const NAME: &'static str = "unyank";

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
