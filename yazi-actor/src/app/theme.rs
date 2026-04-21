use anyhow::Result;
use yazi_actor::Ctx;
use yazi_config::{THEME, build_flavor};
use yazi_emulator::EMULATOR;
use yazi_macro::{render, succ};
use yazi_parser::VoidForm;
use yazi_shared::data::Data;
use yazi_shim::serde::Overlay;

use crate::Actor;

pub struct Theme;

impl Actor for Theme {
	type Form = VoidForm;

	const NAME: &str = "theme";

	fn act(_cx: &mut Ctx, _: Self::Form) -> Result<Data> {
		THEME.overlay(build_flavor(EMULATOR.light, true)?);
		yazi_plugin::theme::reset()?;

		succ!(render!());
	}
}
