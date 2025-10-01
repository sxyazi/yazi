use anyhow::Result;
use yazi_dds::spark::SparkKind;
use yazi_shared::data::Data;

use crate::Ctx;

pub trait Actor {
	type Options;

	const NAME: &str;

	fn act(cx: &mut Ctx, opt: Self::Options) -> Result<Data>;

	fn hook(_cx: &Ctx, _opt: &Self::Options) -> Option<SparkKind> { None }
}
