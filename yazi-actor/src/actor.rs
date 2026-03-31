use anyhow::Result;
use yazi_parser::spark::SparkKind;
use yazi_shared::data::Data;

use crate::Ctx;

pub trait Actor {
	type Form;

	const NAME: &str;

	fn act(cx: &mut Ctx, form: Self::Form) -> Result<Data>;

	fn hook(_cx: &Ctx, _form: &Self::Form) -> Option<SparkKind> { None }
}
