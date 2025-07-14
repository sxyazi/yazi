use anyhow::Result;
use yazi_shared::event::Data;

use crate::Ctx;

pub trait Actor {
	type Options;

	const NAME: &'static str;

	fn act(cx: &mut Ctx, opt: Self::Options) -> Result<Data>;
}
