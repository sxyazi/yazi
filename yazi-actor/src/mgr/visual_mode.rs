use std::collections::BTreeSet;

use anyhow::Result;
use yazi_core::tab::Mode;
use yazi_macro::{render, succ};
use yazi_parser::mgr::VisualModeForm;
use yazi_shared::data::Data;

use crate::{Actor, Ctx};

pub struct VisualMode;

impl Actor for VisualMode {
	type Form = VisualModeForm;

	const NAME: &str = "visual_mode";

	fn act(cx: &mut Ctx, form: Self::Form) -> Result<Data> {
		let tab = cx.tab_mut();

		let idx = tab.current.cursor;
		if form.unset {
			tab.mode = Mode::Unset(idx, BTreeSet::from([idx]));
		} else {
			tab.mode = Mode::Select(idx, BTreeSet::from([idx]));
		};

		succ!(render!());
	}
}
