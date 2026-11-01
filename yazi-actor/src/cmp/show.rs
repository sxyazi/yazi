use std::mem;

use anyhow::Result;
use yazi_core::Ctx;
use yazi_macro::{render, succ};
use yazi_parser::cmp::ShowForm;
use yazi_shared::data::Data;

use crate::Actor;

pub struct Show;

impl Actor for Show {
	type Form = ShowForm;

	const NAME: &str = "show";

	fn act(cx: &mut Ctx, Self::Form { opt }: Self::Form) -> Result<Data> {
		let cmp = &mut cx.cmp;
		if cmp.ticket != opt.ticket {
			succ!();
		}

		if !cmp.refresh(opt) {
			succ!(render!(mem::replace(&mut cmp.visible, false)));
		}

		cmp.offset = 0;
		cmp.cursor = 0;
		cmp.visible = true;
		succ!(render!());
	}
}
