use anyhow::Result;
use yazi_macro::{act, render, succ};
use yazi_parser::ArrowForm;
use yazi_shared::data::Data;

use crate::{Actor, Ctx};

pub struct Arrow;

impl Actor for Arrow {
	type Form = ArrowForm;

	const NAME: &str = "arrow";

	fn act(cx: &mut Ctx, form: Self::Form) -> Result<Data> {
		let tab = cx.tab_mut();
		let old = tab.current.cursor;
		if !tab.current.arrow(form.step) {
			succ!();
		}

		// Retrace
		tab.current.retrace();

		// Visual selection
		if let Some(visual) = tab.mode.visual_mut() {
			visual.arrow(form.step, old, tab.current.cursor);
		}

		act!(mgr:hover, cx)?;
		act!(mgr:peek, cx)?;
		act!(mgr:watch, cx)?;

		cx.tasks.scheduler.behavior.reset();
		succ!(render!());
	}
}
