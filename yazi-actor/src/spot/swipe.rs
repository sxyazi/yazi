use anyhow::Result;
use yazi_macro::act;
use yazi_parser::ArrowForm;
use yazi_shared::data::Data;

use crate::{Actor, Ctx};

pub struct Swipe;

impl Actor for Swipe {
	type Form = ArrowForm;

	const NAME: &str = "swipe";

	fn act(cx: &mut Ctx, form: Self::Form) -> Result<Data> {
		act!(mgr:arrow, cx, form)?;
		act!(mgr:spot, cx)
	}
}
