use anyhow::Result;
use yazi_macro::{act, render, succ};
use yazi_parser::mgr::TabSwitchOpt;
use yazi_shared::event::Data;

use crate::{Actor, Ctx};

pub struct TabSwitch;

impl Actor for TabSwitch {
	type Options = TabSwitchOpt;

	const NAME: &'static str = "tab_switch";

	fn act(cx: &mut Ctx, opt: Self::Options) -> Result<Data> {
		let tabs = cx.tabs_mut();
		let idx = if opt.relative {
			opt.step.saturating_add_unsigned(tabs.cursor).rem_euclid(tabs.len() as _) as _
		} else {
			opt.step as usize
		};

		if idx == tabs.cursor || idx >= tabs.len() {
			succ!();
		}

		tabs.set_idx(idx);
		let cx = &mut Ctx::active(cx.core);

		act!(mgr:refresh, cx)?;
		act!(mgr:peek, cx, true)?;
		succ!(render!());
	}
}
