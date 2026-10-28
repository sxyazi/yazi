use anyhow::Result;
use yazi_core::Ctx;
use yazi_macro::{render, succ};
use yazi_parser::mgr::UpdateSpottedForm;
use yazi_shared::data::Data;

use crate::Actor;

pub struct UpdateSpotted;

impl Actor for UpdateSpotted {
	type Form = UpdateSpottedForm;

	const NAME: &str = "update_spotted";

	fn act(cx: &mut Ctx, mut form: Self::Form) -> Result<Data> {
		let Some(hovered) = cx.hovered_url() else {
			succ!(cx.spot_mut().reset());
		};

		if form.lock.url != *hovered {
			succ!();
		} else if cx.scope.is_cancelled() {
			succ!();
		}

		let spot = &mut cx.spot_mut();
		if spot.lock.as_ref().is_none_or(|l| l.id != form.lock.id) {
			spot.skip = form.lock.selected().unwrap_or_default();
		} else if let Some(s) = form.lock.selected() {
			spot.skip = s;
		} else {
			form.lock.select(Some(spot.skip));
		}

		spot.lock = Some(form.lock);
		succ!(render!());
	}
}
