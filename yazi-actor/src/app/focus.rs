use anyhow::Result;
use yazi_core::Ctx;
use yazi_dds::Pubsub;
use yazi_macro::{log_if_err, succ};
use yazi_parser::app::FocusForm;
use yazi_shared::data::Data;

use crate::{Actor, act};

pub struct Focus;

impl Actor for Focus {
	type Form = FocusForm;

	const NAME: &str = "focus";

	fn act(cx: &mut Ctx, form: Self::Form) -> Result<Data> {
		log_if_err!(Pubsub::pub_after_focus(form.lost));

		if !form.lost {
			act!(mgr:refresh, cx)?;
		}

		succ!()
	}
}
