use anyhow::Result;
use yazi_core::input::InputMutGuard;
use yazi_macro::succ;
use yazi_parser::input::RecallForm;
use yazi_shared::data::Data;

use crate::{Actor, Ctx};

pub struct Recall;

impl Actor for Recall {
	type Form = RecallForm;

	const NAME: &str = "recall";

	fn act(cx: &mut Ctx, form: Self::Form) -> Result<Data> {
		let Some(input) = cx.input.lock_mut() else {
			succ!();
		};

		match input {
			InputMutGuard::Main(input) => {
				let entries = input.histories.get(&input.main.history.name);
				input.main.recall(entries, form.step)
			}
			InputMutGuard::Alt(input, mut guard) => {
				let entries = input.histories.get(&guard.history.name);
				guard.recall(entries, form.step)
			}
		}
	}
}
