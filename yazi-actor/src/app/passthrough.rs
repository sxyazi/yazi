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
		if cx.term.is_none() {
			succ!();
		}

		if EMULATOR.probe.id != id || !EMULATOR.needs_passthrough() {
			succ!();
		}

		if let Err(e) = EMULATOR.restart() {
			error!("Failed to request terminal capabilities through tmux: {e}");
		} else {
			render_force!();
		}

		succ!();
	}
}
