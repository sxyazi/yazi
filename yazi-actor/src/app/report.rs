use anyhow::Result;
use yazi_adapter::ADAPTOR;
use yazi_emulator::{EMULATOR, Mux};
use yazi_macro::{act, succ};
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

		if report.is_color_scheme()
			&& old_light.zip(EMULATOR.light()).is_some_and(|(a, b)| a != b)
			&& !EMULATOR.needs_passthrough()
		{
			return act!(app:theme, cx);
		} else if !report.is_da_1() {
			succ!();
		} else if !EMULATOR.needs_passthrough() {
			ADAPTOR.resolve(&EMULATOR);
			return act!(app:theme, cx);
		}

		let id = EMULATOR.probe_id.get();
		tokio::spawn(async move {
			Mux::tmux_setup().await;
			AppProxy::passthrough(id);
		});

		succ!();
	}
}
