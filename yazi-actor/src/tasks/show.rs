use anyhow::Result;
use yazi_macro::{act, render, succ};
use yazi_parser::VoidOpt;
use yazi_shared::data::Data;

use crate::{Actor, Ctx};

pub struct Show;

impl Actor for Show {
	type Options = VoidOpt;

	const NAME: &str = "show";

	fn act(cx: &mut Ctx, _: Self::Options) -> Result<Data> {
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
