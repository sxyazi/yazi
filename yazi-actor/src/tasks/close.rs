use anyhow::Result;
use yazi_macro::{act, render, succ};
use yazi_parser::VoidOpt;
use yazi_shared::data::Data;

use crate::{Actor, Ctx};

pub struct Close;

impl Actor for Close {
	type Options = VoidOpt;

	const NAME: &str = "close";

	fn act(cx: &mut Ctx, _: Self::Options) -> Result<Data> {
		let tasks = &mut cx.tasks;
		if !tasks.visible {
			succ!();
		}

		tasks.visible = false;
		tasks.snaps = Vec::new();

		act!(tasks:arrow, cx)?;
		succ!(render!());
	}
}
