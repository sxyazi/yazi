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

		term.probe.emulator.apply(&report);
		EMULATOR.store(Arc::new(term.probe.emulator.clone()));

		if !matches!(report, TermReport::Da1(_)) {
			succ!();
		} else if !term.probe.needs_passthrough() {
			ADAPTOR.resolve(&term.probe.emulator);
			return act!(app:theme, cx);
		}

		let id = term.probe.id;
		tokio::spawn(async move {
			Mux::tmux_passthrough().await;
			AppProxy::passthrough(id);
		});

		succ!();
	}
}
