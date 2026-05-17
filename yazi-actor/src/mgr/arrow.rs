use anyhow::Result;
use yazi_dds::Pubsub;
use yazi_macro::{act, err, render, succ};
use yazi_parser::ArrowForm;
use yazi_shared::data::Data;

use crate::{Actor, Ctx};

pub struct Arrow;

impl Actor for Arrow {
	type Form = ArrowForm;

	const NAME: &str = "arrow";

	fn act(cx: &mut Ctx, form: Self::Form) -> Result<Data> {
		let window_relative = form.step.is_window_relative();
		let tab = cx.tab_mut();
		if !tab.current.arrow(form.step) {
			succ!();
		}

		// Retrace
		tab.current.retrace();

		// Visual selection
		if let Some((start, items)) = tab.mode.visual_mut() {
			let end = tab.current.cursor;
			*items = (start.min(end)..=end.max(start)).collect();
		}

		if window_relative {
			err!(Pubsub::pub_after_hover(tab.id, tab.hovered().map(|h| &h.url)));
		} else {
			act!(mgr:hover, cx)?;
		}
		act!(mgr:peek, cx)?;
		act!(mgr:watch, cx)?;

		cx.tasks.scheduler.behavior.reset();
		succ!(render!());
	}
}
