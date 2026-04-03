use anyhow::Result;
use yazi_actor::Ctx;
use yazi_macro::{act, render, render_partial, succ};
use yazi_parser::app::UpdateProgressForm;
use yazi_shared::data::Data;

use crate::Actor;

pub struct UpdateProgress;

impl Actor for UpdateProgress {
	type Form = UpdateProgressForm;

	const NAME: &str = "update_progress";

	fn act(cx: &mut Ctx, form: Self::Form) -> Result<Data> {
		// Update the progress of all tasks.
		let tasks = &mut cx.tasks;
		let progressed = tasks.summary != form.summary;
		tasks.summary = form.summary;

		// If the task manager is visible, update the snaps with a full render.
		if tasks.visible {
			let new = tasks.paginate();
			if tasks.snaps != new {
				tasks.snaps = new;
				act!(tasks:arrow, cx)?;
				succ!(render!());
			}
		}

		if !progressed {
			succ!()
		} else if tasks.summary.total == 0 {
			succ!(render!())
		} else {
			succ!(render_partial!())
		}
	}
}
