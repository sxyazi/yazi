use anyhow::Result;
use yazi_macro::succ;
use yazi_parser::VoidForm;
use yazi_shared::data::Data;

use crate::{Actor, Ctx};

pub struct Dismiss;

impl Actor for Dismiss {
	type Form = VoidForm;

	const NAME: &str = "dismiss";

	fn act(cx: &mut Ctx, _: Self::Form) -> Result<Data> {
		succ!(cx.which.dismiss(None));
	}
}
