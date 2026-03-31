use anyhow::Result;
use yazi_macro::succ;
use yazi_parser::VoidForm;
use yazi_shared::data::Data;

use crate::{Actor, Ctx};

pub struct Close;

impl Actor for Close {
	type Form = VoidForm;

	const NAME: &str = "close";

	fn act(cx: &mut Ctx, _: Self::Form) -> Result<Data> {
		succ!(cx.tab_mut().spot.reset());
	}
}
