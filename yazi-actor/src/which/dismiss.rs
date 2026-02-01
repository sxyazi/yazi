use anyhow::Result;
use yazi_macro::succ;
use yazi_parser::VoidOpt;
use yazi_shared::data::Data;

use crate::{Actor, Ctx};

pub struct Dismiss;

impl Actor for Dismiss {
	type Options = VoidOpt;

	const NAME: &str = "dismiss";

	fn act(cx: &mut Ctx, _: Self::Options) -> Result<Data> {
		succ!(cx.which.dismiss(None));
	}
}
