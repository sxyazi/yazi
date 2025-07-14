use anyhow::Result;
use yazi_macro::{act, succ};
use yazi_parser::{VoidOpt, tab::CdSource};
use yazi_shared::event::Data;

use crate::{Actor, Ctx};

pub struct Forward;

impl Actor for Forward {
	type Options = VoidOpt;

	const NAME: &'static str = "forward";

	fn act(cx: &mut Ctx, _: Self::Options) -> Result<Data> {
		match cx.tab_mut().backstack.shift_forward().cloned() {
			Some(u) => act!(mgr:cd, cx, (u, CdSource::Forward)),
			None => succ!(),
		}
	}
}
