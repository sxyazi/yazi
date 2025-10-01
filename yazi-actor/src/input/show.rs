use std::ops::DerefMut;

use anyhow::Result;
use yazi_config::YAZI;
use yazi_macro::{act, render, succ};
use yazi_parser::input::ShowOpt;
use yazi_shared::{data::Data, errors::InputError};
use yazi_widgets::input::InputCallback;

use crate::{Actor, Ctx};

pub struct Show;

impl Actor for Show {
	type Options = ShowOpt;

	const NAME: &str = "show";

	fn act(cx: &mut Ctx, opt: Self::Options) -> Result<Data> {
		act!(input:close, cx)?;

		let input = &mut cx.input;
		input.visible = true;
		input.title = opt.cfg.title;
		input.position = opt.cfg.position;

		// Typing
		input.tx = Some(opt.tx.clone());
		let ticket = input.ticket.clone();

		// Reset input
		let cb: InputCallback = Box::new(move |before, after| {
			if opt.cfg.realtime {
				opt.tx.send(Err(InputError::Typed(format!("{before}{after}")))).ok();
			} else if opt.cfg.completion {
				opt.tx.send(Err(InputError::Completed(before.to_owned(), ticket.current()))).ok();
			}
		});
		*input.deref_mut() = yazi_widgets::input::Input::new(
			opt.cfg.value,
			opt.cfg.position.offset.width.saturating_sub(YAZI.input.border()) as usize,
			opt.cfg.obscure,
			cb,
		);

		// Set cursor after reset
		// TODO: remove this
		if let Some(cursor) = opt.cfg.cursor {
			input.snap_mut().cursor = cursor;
			act!(r#move, input)?;
		}

		succ!(render!());
	}
}
