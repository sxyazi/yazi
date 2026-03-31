use anyhow::Result;
use yazi_macro::{act, render, succ};
use yazi_parser::VoidForm;
use yazi_shared::data::Data;

use crate::{Actor, Ctx};

pub struct Show;

impl Actor for Show {
	type Form = VoidForm;

	const NAME: &str = "show";

	fn act(cx: &mut Ctx, _: Self::Form) -> Result<Data> {
		let tasks = &mut cx.tasks;
		if tasks.visible {
			succ!();
		}

		tasks.visible = true;
		tasks.snaps = tasks.paginate();

		act!(tasks:arrow, cx)?;
		succ!(render!());
	}
}
