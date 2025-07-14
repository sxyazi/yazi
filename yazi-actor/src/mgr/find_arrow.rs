use anyhow::Result;
use yazi_macro::{act, render, succ};
use yazi_parser::tab::FindArrowOpt;
use yazi_shared::event::Data;

use crate::{Actor, Ctx};

pub struct FindArrow;

impl Actor for FindArrow {
	type Options = FindArrowOpt;

	const NAME: &'static str = "find_arrow";

	fn act(cx: &mut Ctx, opt: Self::Options) -> Result<Data> {
		let tab = cx.tab_mut();
		let Some(finder) = &mut tab.finder else { succ!() };

		render!(finder.catchup(&tab.current));
		let offset = if opt.prev {
			finder.prev(&tab.current.files, tab.current.cursor, false)
		} else {
			finder.next(&tab.current.files, tab.current.cursor, false)
		};

		if let Some(offset) = offset {
			act!(mgr:arrow, cx, offset)?;
		}
		succ!();
	}
}
