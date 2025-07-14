use anyhow::Result;
use yazi_macro::{act, render, succ};
use yazi_parser::VoidOpt;
use yazi_shared::event::Data;

use crate::{Actor, Ctx};

pub struct Cancel;

impl Actor for Cancel {
	type Options = VoidOpt;

	const NAME: &'static str = "cancel";

	fn act(cx: &mut Ctx, _: Self::Options) -> Result<Data> {
		let tasks = &mut cx.tasks;

		let id = tasks.ongoing().lock().get_id(tasks.cursor);
		if id.map(|id| tasks.scheduler.cancel(id)) != Some(true) {
			succ!();
		}

		tasks.summaries = tasks.paginate();
		act!(tasks:arrow, cx)?;
		succ!(render!());
	}
}
