use anyhow::Result;
use yazi_macro::act;
use yazi_parser::ArrowOpt;
use yazi_shared::data::Data;

use crate::{Actor, Ctx};

pub struct Swipe;

impl Actor for Swipe {
	type Options = ArrowOpt;

	const NAME: &str = "swipe";

	fn act(cx: &mut Ctx, opt: Self::Options) -> Result<Data> {
		act!(mgr:arrow, cx, opt)?;
		act!(mgr:spot, cx)
	}
}
