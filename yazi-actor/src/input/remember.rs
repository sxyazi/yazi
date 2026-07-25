use anyhow::Result;
use yazi_core::input::InputMutGuard;
use yazi_macro::succ;
use yazi_parser::VoidForm;
use yazi_shared::data::Data;

use crate::{Actor, Ctx};

pub struct Remember;

impl Actor for Remember {
	type Form = VoidForm;

	const NAME: &str = "remember";

	fn act(cx: &mut Ctx, _: Self::Form) -> Result<Data> {
		let Some(mut input) = cx.input.lock_mut() else {
			succ!();
		};

		match &mut input {
			InputMutGuard::Main(input) => {
				input.histories.remember(&input.main.history.name, input.main.value());
			}
			InputMutGuard::Alt(input, guard) => {
				input.histories.remember(&guard.history.name, guard.value());
			}
		}

		input.history.take();
		succ!();
	}
}
