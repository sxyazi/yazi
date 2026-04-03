use anyhow::Result;
use yazi_macro::{act, render, succ};
use yazi_parser::mgr::TabSwitchForm;
use yazi_shared::data::Data;

use crate::{Actor, Ctx};

pub struct TabSwitch;

impl Actor for TabSwitch {
	type Form = TabSwitchForm;

	const NAME: &str = "tab_switch";

	fn act(cx: &mut Ctx, form: Self::Form) -> Result<Data> {
		let tabs = cx.tabs_mut();
		let idx = if form.relative {
			form.step.saturating_add_unsigned(tabs.cursor).rem_euclid(tabs.len() as _) as _
		} else {
			form.step as usize
		};

		if idx == tabs.cursor || idx >= tabs.len() {
			succ!();
		}

		tabs.set_idx(idx);
		let cx = &mut Ctx::renew(cx);

		act!(mgr:refresh, cx)?;
		act!(mgr:peek, cx, true)?;
		act!(app:title, cx).ok();
		succ!(render!());
	}
}
