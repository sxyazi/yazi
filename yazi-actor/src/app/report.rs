use std::sync::Arc;

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
		let Some(term) = cx.term.as_mut() else { succ!() };
		let old_light = term.probe.emulator.light();

		term.probe.emulator.apply(&report);
		EMULATOR.store(Arc::new(term.probe.emulator.clone()));

		if report.is_color_scheme()
			&& old_light.zip(term.probe.emulator.light()).is_some_and(|(a, b)| a != b)
			&& !term.probe.needs_passthrough()
		{
			return act!(app:theme, cx);
		} else if !report.is_da_1() {
			succ!();
		} else if !term.probe.needs_passthrough() {
			term.clear().ok();
			ADAPTOR.resolve(&term.probe.emulator);
			return act!(app:theme, cx);
		}

		let id = term.probe.id;
		tokio::spawn(async move {
			Mux::tmux_setup().await;
			AppProxy::passthrough(id);
		});

		succ!();
	}
}
