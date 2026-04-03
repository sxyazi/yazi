use anyhow::Result;
use yazi_macro::succ;
use yazi_parser::mgr::BulkExitForm;
use yazi_shared::data::Data;

use crate::{Actor, Ctx};

pub struct BulkExit;

impl Actor for BulkExit {
	type Form = BulkExitForm;

	const NAME: &str = "bulk_exit";

	fn act(cx: &mut Ctx, form: Self::Form) -> Result<Data> {
		cx.mgr.batcher.decide(form.target, form.accept);
		succ!();
	}
}
