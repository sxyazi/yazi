use anyhow::Result;
use yazi_macro::{emit, render, succ};
use yazi_parser::help::CloseForm;
use yazi_shared::data::Data;

use crate::{Actor, Ctx};

pub struct Close;

impl Actor for Close {
	type Form = CloseForm;

	const NAME: &str = "close";

	fn act(cx: &mut Ctx, opt: Self::Form) -> Result<Data> {
		let help = &mut cx.help;

		if let Some(chord) = help.bindings.get(help.cursor).filter(|_| opt.submit) {
			emit!(Seq(chord.to_seq(help.layer)));
		}

		help.visible = false;
		succ!(render!());
	}
}
