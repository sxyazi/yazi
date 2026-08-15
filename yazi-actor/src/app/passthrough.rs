use std::sync::Arc;

use anyhow::Result;
use yazi_emulator::EMULATOR;
use yazi_macro::{error, render_force, succ};
use yazi_parser::app::PassthroughForm;
use yazi_shared::data::Data;

use crate::{Actor, Ctx};

pub struct Passthrough;

impl Actor for Passthrough {
	type Form = PassthroughForm;

	const NAME: &str = "passthrough";

	fn act(cx: &mut Ctx, PassthroughForm { id }: Self::Form) -> Result<Data> {
		let Some(term) = cx.term.as_mut() else { succ!() };

		if term.probe.id != id || !term.probe.needs_passthrough() {
			succ!();
		}

		if let Err(e) = term.probe.restart() {
			error!("Failed to request terminal capabilities through tmux: {e}");
		} else {
			render_force!();
		}

		EMULATOR.store(Arc::new(term.probe.emulator.clone()));
		succ!();
	}
}
