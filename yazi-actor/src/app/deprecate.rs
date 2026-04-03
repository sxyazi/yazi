use anyhow::Result;
use yazi_core::notify::{MessageLevel, MessageOpt};
use yazi_macro::act;
use yazi_parser::app::DeprecateForm;
use yazi_shared::data::Data;

use crate::{Actor, Ctx};

pub struct Deprecate;

impl Actor for Deprecate {
	type Form = DeprecateForm;

	const NAME: &str = "deprecate";

	fn act(cx: &mut Ctx, form: Self::Form) -> Result<Data> {
		act!(notify:push, cx, MessageOpt {
			title:   "Deprecated API".to_owned(),
			content: form.content.into_owned(),
			level:   MessageLevel::Warn,
			timeout: std::time::Duration::from_secs(20),
		})
	}
}
