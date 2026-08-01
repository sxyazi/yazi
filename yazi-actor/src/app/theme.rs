use anyhow::Result;
use yazi_actor::Ctx;
use yazi_config::{THEME, build_flavor};
use yazi_emulator::EMULATOR;
use yazi_macro::{act, render, succ};
use yazi_parser::VoidForm;
use yazi_scheduler::NotifyProxy;
use yazi_shared::data::Data;
use yazi_shim::serde::Overlay;

use crate::Actor;

pub struct Theme;

impl Actor for Theme {
	type Form = VoidForm;

	const NAME: &str = "theme";

	fn act(cx: &mut Ctx, _: Self::Form) -> Result<Data> {
		match build_flavor(EMULATOR.load().light().unwrap_or_default()) {
			Ok(theme) => THEME.overlay(theme),
			Err(e) => succ!(NotifyProxy::push_error("Theme load failed", format!("{e:#}"))),
		};

		yazi_plugin::theme::reset()?;

		act!(mgr:peek, cx, true)?;
		if cx.tab().spot.visible() {
			act!(mgr:spot, cx, true)?;
		}

		succ!(render!());
	}
}
