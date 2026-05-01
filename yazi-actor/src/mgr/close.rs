use anyhow::Result;
use yazi_macro::act;
use yazi_parser::{mgr::CloseForm, spark::SparkKind};
use yazi_shared::data::Data;

use crate::{Actor, Ctx};

pub struct Close;

impl Actor for Close {
	type Form = CloseForm;

	const NAME: &str = "close";

	fn act(cx: &mut Ctx, Self::Form { opt }: Self::Form) -> Result<Data> {
		if cx.tabs().len() > 1 {
			act!(mgr:tab_close, cx, cx.tabs().cursor)
		} else {
			act!(mgr:quit, cx, opt)
		}
	}

	fn hook(cx: &Ctx, _form: &Self::Form) -> Option<SparkKind> {
		Some(SparkKind::KeyClose).filter(|_| cx.source().is_key())
	}
}
