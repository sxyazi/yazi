use anyhow::Result;
use yazi_core::input::InputMutGuard;
use yazi_macro::{act, render, succ};
use yazi_parser::{input::CloseForm, spark::SparkKind};
use yazi_shared::{Source, data::Data};
use yazi_widgets::input::InputEvent;

use crate::{Actor, Ctx};

pub struct Close;

impl Actor for Close {
	type Form = CloseForm;

	const NAME: &str = "close";

	fn act(cx: &mut Ctx, form: Self::Form) -> Result<Data> {
		let Some(mut guard) = cx.input.lock_mut() else {
			succ!();
		};

		guard.ticket.next();
		if let Some(cb) = guard.cb.take() {
			let value = guard.value().to_owned();
			cb(if form.submit { InputEvent::Submit(value) } else { InputEvent::Cancel(value) });
		}

		if form.submit
			&& let InputMutGuard::Main(input) = guard
		{
			input.histories.remember(&input.main.history.name, input.main.value());
		}

		cx.input.main.visible = false;

		act!(cmp:close, cx)?;
		succ!(render!());
	}

	fn hook(cx: &Ctx, _form: &Self::Form) -> Option<SparkKind> {
		match cx.source() {
			Source::Key => Some(SparkKind::KeyInputClose),
			Source::Ind => Some(SparkKind::IndInputClose),
			_ => None,
		}
	}
}
