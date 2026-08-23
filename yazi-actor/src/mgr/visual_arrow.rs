use anyhow::Result;
use yazi_macro::{act, render, succ};
use yazi_parser::mgr::VisualArrowForm;
use yazi_shared::data::Data;

use crate::{Actor, Ctx};

pub struct VisualArrow;

impl Actor for VisualArrow {
	type Form = VisualArrowForm;

	const NAME: &str = "visual_arrow";

	fn act(cx: &mut Ctx, form: Self::Form) -> Result<Data> {
		let tab = cx.tab_mut();
		let len = tab.current.entries.len() as i128;
		if len == 0 {
			succ!();
		}

		let Some(visual) = tab.mode.visual_mut() else { succ!() };
		let new = tab.current.cursor as i128 + form.step as i128;

		visual.start = tab.current.cursor;
		visual.wraps = new.div_euclid(len) as isize;
		tab.current.cursor = new.rem_euclid(len) as usize;

		tab.current.arrow(0);
		tab.current.retrace();

		act!(mgr:hover, cx)?;
		act!(mgr:peek, cx)?;
		act!(mgr:watch, cx).ok();

		cx.tasks.scheduler.behavior.reset();
		succ!(render!());
	}
}
