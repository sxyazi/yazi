use anyhow::Result;
use yazi_core::tasks::Tasks;
use yazi_macro::{render, succ};
use yazi_parser::ArrowOpt;
use yazi_shared::event::Data;

use crate::{Actor, Ctx};

pub struct Arrow;

impl Actor for Arrow {
	type Options = ArrowOpt;

	const NAME: &'static str = "arrow";

	fn act(cx: &mut Ctx, opt: Self::Options) -> Result<Data> {
		let tasks = &mut cx.tasks;

		let old = tasks.cursor;
		tasks.cursor = opt.step.add(tasks.cursor, tasks.summaries.len(), Tasks::limit());

		succ!(render!(tasks.cursor != old));
	}
}
