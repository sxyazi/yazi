use anyhow::Result;
use yazi_core::Ctx;
use yazi_macro::succ;
use yazi_shared::data::Data;
use yazi_vfs::Completer;
use yazi_widgets::input::parser::CompleteOpt;

use crate::{Actor, act};

pub struct Complete;

impl Actor for Complete {
	type Form = CompleteOpt;

	const NAME: &str = "complete";

	fn act(cx: &mut Ctx, mut form: Self::Form) -> Result<Data> {
		let Some(mut guard) = cx.input.lock_mut() else {
			succ!();
		};

		if guard.ticket.current() != form.ticket {
			succ!();
		}

		// Resolve the completion value.
		let completer = Completer::new(guard.partition().0);
		let (key, _) = completer.parse()?;
		if let Some(new) = completer.resolve(&key, &form) {
			form.value = Some(new);
		}

		act!(complete, guard, form)
	}
}
