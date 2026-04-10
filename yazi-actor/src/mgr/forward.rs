use anyhow::Result;
use yazi_core::mgr::CdSource;
use yazi_macro::{act, succ};
use yazi_parser::VoidForm;
use yazi_shared::data::Data;

use crate::{Actor, Ctx};

pub struct Forward;

impl Actor for Forward {
	type Form = VoidForm;

	const NAME: &str = "forward";

	fn act(cx: &mut Ctx, _: Self::Form) -> Result<Data> {
		if let Some(u) = cx.tab_mut().backstack.shift_forward().cloned() {
			act!(mgr:cd, cx, (u, CdSource::Forward))?;
		}
		succ!()
	}
}
