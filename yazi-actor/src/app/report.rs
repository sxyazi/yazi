use anyhow::Result;
use yazi_adapter::ADAPTOR;
use yazi_emulator::{EMULATOR, Mux};
use yazi_macro::{act, log_if_err, succ};
use yazi_proxy::AppProxy;
use yazi_shared::data::Data;
use yazi_term::event::Report as TermReport;

use crate::{Actor, Ctx};

pub struct Report;

impl Actor for Report {
	type Form = TermReport;

	const NAME: &str = "report";

	fn act(cx: &mut Ctx, report: Self::Form) -> Result<Data> {
		if cx.term.is_none() {
			succ!();
		}

		let old_light = EMULATOR.light();
		EMULATOR.apply(&report);

		if EMULATOR.light() != old_light {
			log_if_err!(act!(app:theme, cx));
		}

		if !report.is_da_1() {
			succ!();
		} else if EMULATOR.needs_passthrough() {
			succ!(Self::reprobe());
		}

		ADAPTOR.resolve(&EMULATOR);
		EMULATOR.probe.complete();
		if EMULATOR.light().is_none() {
			log_if_err!(act!(app:theme, cx));
		}

		succ!();
	}
}

impl Report {
	fn reprobe() {
		let id = EMULATOR.probe.id.get();
		tokio::spawn(async move {
			Mux::tmux_setup().await;
			AppProxy::passthrough(id);
		});
	}
}
