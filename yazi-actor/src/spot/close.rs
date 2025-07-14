use anyhow::Result;
use yazi_macro::succ;
use yazi_parser::VoidOpt;
use yazi_shared::event::Data;

use crate::{Actor, Ctx};

pub struct Close;

impl Actor for Close {
	type Options = VoidOpt;

	const NAME: &'static str = "close";

	fn act(cx: &mut Ctx, _: Self::Options) -> Result<Data> {
		succ!(cx.tab_mut().spot.reset());
	}
}
