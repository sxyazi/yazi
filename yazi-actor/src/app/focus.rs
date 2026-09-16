use anyhow::Result;
use yazi_actor::Ctx;
use yazi_parser::VoidForm;
use yazi_shared::data::Data;

use crate::{Actor, act};

pub struct Focus;

impl Actor for Focus {
	type Form = VoidForm;

	const NAME: &str = "focus";

	fn act(cx: &mut Ctx, _: Self::Form) -> Result<Data> { act!(mgr:refresh, cx) }
}
