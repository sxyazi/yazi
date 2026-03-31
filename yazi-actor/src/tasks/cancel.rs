use anyhow::Result;
use yazi_macro::{act, render, succ};
use yazi_parser::VoidForm;
use yazi_shared::data::Data;

use crate::{Actor, Ctx};

pub struct Cancel;

impl Actor for Cancel {
	type Form = VoidForm;

	const NAME: &str = "cancel";

	fn act(cx: &mut Ctx, _: Self::Form) -> Result<Data> {
		let tasks = &mut cx.tasks;

		let id = tasks.scheduler.ongoing.lock().get_id(tasks.cursor);
		if id.map(|id| tasks.scheduler.cancel(id)) != Some(true) {
			succ!();
		}

		tasks.snaps = tasks.paginate();
		act!(tasks:arrow, cx)?;
		succ!(render!());
	}
}
