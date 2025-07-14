use anyhow::Result;
use yazi_macro::{act, succ};
use yazi_parser::{VoidOpt, tab::CdSource};
use yazi_shared::event::Data;

use crate::{Actor, Ctx};

pub struct Back;

impl Actor for Back {
	type Options = VoidOpt;

	const NAME: &'static str = "back";

	fn act(cx: &mut Ctx, _: Self::Options) -> Result<Data> {
		if let Some(u) = cx.tab_mut().backstack.shift_backward().cloned() {
			return act!(mgr:cd, cx, (u, CdSource::Back));
		}
		succ!();
	}
}
