use anyhow::Result;
use yazi_macro::succ;
use yazi_parser::mgr::SpotOpt;
use yazi_shared::data::Data;

use crate::{Actor, Ctx};

pub struct Spot;

impl Actor for Spot {
	type Options = SpotOpt;

	const NAME: &str = "spot";

	fn act(cx: &mut Ctx, opt: Self::Options) -> Result<Data> {
		let Some(hovered) = cx.hovered().cloned() else { succ!() };

		let mime = cx.mgr.mimetype.owned(&hovered.url).unwrap_or_default();
		// if !self.active().spot.same_file(&hovered, &mime) {
		// self.active_mut().spot.reset();
		// }

		if let Some(skip) = opt.skip {
			cx.tab_mut().spot.skip = skip;
		} else if !cx.tab().spot.same_url(&hovered.url) {
			cx.tab_mut().spot.skip = 0;
		}

		cx.tab_mut().spot.go(hovered, mime);
		succ!();
	}
}
