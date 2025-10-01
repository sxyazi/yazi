use anyhow::Result;
use yazi_macro::{act, render, succ};
use yazi_parser::ArrowOpt;
use yazi_shared::data::Data;

use crate::{Actor, Ctx};

pub struct Arrow;

impl Actor for Arrow {
	type Options = ArrowOpt;

	const NAME: &str = "arrow";

	fn act(cx: &mut Ctx, opt: Self::Options) -> Result<Data> {
		let spot = &mut cx.tab_mut().spot;
		let Some(lock) = &mut spot.lock else { succ!() };

		let new = opt.step.add(spot.skip, lock.len().unwrap_or(u16::MAX as _), 0);
		let Some(old) = lock.selected() else {
			return act!(mgr:spot, cx, new);
		};

		lock.select(Some(new));
		let new = lock.selected().unwrap();

		spot.skip = new;
		succ!(render!(new != old));
	}
}
