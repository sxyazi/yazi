use anyhow::Result;
use yazi_actor::Ctx;
use yazi_macro::act;
use yazi_parser::VoidForm;
use yazi_shared::data::Data;

use crate::Actor;

pub struct Focus;

impl Actor for Focus {
	type Options = VoidForm;

	const NAME: &str = "focus";

	fn act(cx: &mut Ctx, _: Self::Options) -> Result<Data> { act!(mgr:refresh, cx) }
}
