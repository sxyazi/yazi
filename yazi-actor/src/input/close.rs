use anyhow::Result;
use yazi_core::{Ctx, input::InputMutGuard};
use yazi_macro::{render, succ};
use yazi_parser::{input::CloseForm, spark::SparkKind};
use yazi_shared::{Source, data::Data};
use yazi_widgets::input::InputEvent;

use crate::{Actor, act};

pub struct Close;

impl Actor for Close {
	type Form = CloseForm;

	const NAME: &str = "close";

	fn act(cx: &mut Ctx, form: Self::Form) -> Result<Data> {
		let Some(guard) = cx.input.lock_mut() else {
			succ!();
		};

		guard.ticket.next();
		if let Some(cb) = guard.cb.as_ref() {
			let value = guard.value().to_owned();
			cb(if form.submit { InputEvent::Submit(value) } else { InputEvent::Cancel(value) });
		}

		if let InputMutGuard::Main(input) = guard {
			if form.submit {
				input.histories.remember(&input.main.history.name, input.main.value());
			}

			*input.main = Default::default();
			input.main.visible = false;
		}

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
