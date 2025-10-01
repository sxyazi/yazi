use anyhow::Result;
use yazi_macro::{render, succ};
use yazi_parser::mgr::UpdateSpottedOpt;
use yazi_shared::data::Data;

use crate::{Actor, Ctx};

pub struct UpdateSpotted;

impl Actor for UpdateSpotted {
	type Options = UpdateSpottedOpt;

	const NAME: &str = "update_spotted";

	fn act(cx: &mut Ctx, mut opt: Self::Options) -> Result<Data> {
		let tab = cx.tab_mut();
		let Some(hovered) = tab.hovered().map(|h| &h.url) else {
			succ!(tab.spot.reset());
		};

		if opt.lock.url != *hovered {
			succ!();
		}

		if tab.spot.lock.as_ref().is_none_or(|l| l.id != opt.lock.id) {
			tab.spot.skip = opt.lock.selected().unwrap_or_default();
		} else if let Some(s) = opt.lock.selected() {
			tab.spot.skip = s;
		} else {
			opt.lock.select(Some(tab.spot.skip));
		}

		tab.spot.lock = Some(opt.lock);
		succ!(render!());
	}
}
