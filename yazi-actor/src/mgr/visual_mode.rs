use std::collections::BTreeSet;

use anyhow::Result;
use yazi_core::tab::Mode;
use yazi_macro::{render, succ};
use yazi_parser::tab::VisualModeOpt;
use yazi_shared::event::Data;

use crate::{Actor, Ctx};

pub struct VisualMode;

impl Actor for VisualMode {
	type Options = VisualModeOpt;

	const NAME: &'static str = "visual_mode";

	fn act(cx: &mut Ctx, opt: Self::Options) -> Result<Data> {
		let tab = cx.tab_mut();

		let idx = tab.current.cursor;
		if opt.unset {
			tab.mode = Mode::Unset(idx, BTreeSet::from([idx]));
		} else {
			tab.mode = Mode::Select(idx, BTreeSet::from([idx]));
		};

		succ!(render!());
	}
}
