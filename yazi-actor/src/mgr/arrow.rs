use anyhow::Result;
use yazi_macro::{act, render, succ};
use yazi_parser::ArrowOpt;
use yazi_shared::data::Data;

use crate::{Actor, Ctx};

pub struct Arrow;

impl Actor for Arrow {
	type Options = ArrowOpt;

	const NAME: &str = "arrow";

	fn act(cx: &mut Ctx, opt: Self::Options) -> Result<Data> {
		let tab = cx.tab_mut();
		if !tab.current.arrow(opt.step) {
			succ!();
		}

		// Visual selection
		if let Some((start, items)) = tab.mode.visual_mut() {
			let end = tab.current.cursor;
			*items = (start.min(end)..=end.max(start)).collect();
		}

		act!(mgr:hover, cx)?;
		act!(mgr:peek, cx)?;
		act!(mgr:watch, cx)?;

		succ!(render!());
	}
}
