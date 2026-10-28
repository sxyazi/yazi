use anyhow::Result;
use yazi_core::Ctx;
use yazi_macro::succ;
use yazi_parser::mgr::SpotOpt;
use yazi_shared::{data::Data, pool::InternStr};

use crate::{Actor, act};

pub struct Spot;

impl Actor for Spot {
	type Form = SpotOpt;

	const NAME: &str = "spot";

	fn act(cx: &mut Ctx, form: Self::Form) -> Result<Data> {
		act!(mgr:escape_visual, cx)?;
		let Some(hovered) = cx.hovered().cloned() else { succ!() };

		let mime = if cx.tab().selected.is_empty() {
			cx.mgr.mimetype.owned(&hovered.url).unwrap_or_default()
		} else {
			"multi/unknown".intern()
		};

		if let Some(skip) = form.skip {
			cx.spot_mut().skip = skip;
		} else if !cx.spot().same_url(&hovered.url) {
			cx.spot_mut().skip = 0;
		}

		cx.spot_mut().go(hovered, mime, form.force);

		succ!();
	}
}
