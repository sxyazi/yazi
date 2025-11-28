use anyhow::Result;
use yazi_macro::{act, succ};
use yazi_parser::{VoidOpt, mgr::CdSource};
use yazi_shared::data::Data;

use crate::{Actor, Ctx};

pub struct Forward;

impl Actor for Forward {
	type Options = VoidOpt;

	const NAME: &str = "forward";

	fn act(cx: &mut Ctx, _: Self::Options) -> Result<Data> {
		if let Some(u) = cx.tab_mut().backstack.shift_forward().cloned() {
			act!(mgr:cd, cx, (u, CdSource::Forward))?;
		}
		succ!()
	}
}
