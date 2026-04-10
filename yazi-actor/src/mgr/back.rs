use anyhow::Result;
use yazi_core::mgr::CdSource;
use yazi_macro::{act, succ};
use yazi_parser::VoidForm;
use yazi_shared::data::Data;

use crate::{Actor, Ctx};

pub struct Back;

impl Actor for Back {
	type Form = VoidForm;

	const NAME: &str = "back";

	fn act(cx: &mut Ctx, _: Self::Form) -> Result<Data> {
		if let Some(u) = cx.tab_mut().backstack.shift_backward().cloned() {
			act!(mgr:cd, cx, (u, CdSource::Back))?;
		}
		succ!();
	}
}
