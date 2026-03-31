use anyhow::Result;
use yazi_macro::{render, succ};
use yazi_parser::mgr::UpdateSpottedForm;
use yazi_shared::data::Data;

use crate::{Actor, Ctx};

pub struct UpdateSpotted;

impl Actor for UpdateSpotted {
	type Form = UpdateSpottedForm;

	const NAME: &str = "update_spotted";

	fn act(cx: &mut Ctx, mut form: Self::Form) -> Result<Data> {
		let tab = cx.tab_mut();
		let Some(hovered) = tab.hovered().map(|h| &h.url) else {
			succ!(tab.spot.reset());
		};

		if form.lock.url != *hovered {
			succ!();
		}

		if tab.spot.lock.as_ref().is_none_or(|l| l.id != form.lock.id) {
			tab.spot.skip = form.lock.selected().unwrap_or_default();
		} else if let Some(s) = form.lock.selected() {
			tab.spot.skip = s;
		} else {
			form.lock.select(Some(tab.spot.skip));
		}

		tab.spot.lock = Some(form.lock);
		succ!(render!());
	}
}
