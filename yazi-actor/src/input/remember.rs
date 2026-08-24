use anyhow::Result;
use yazi_core::input::InputMutGuard;
use yazi_dds::Pubsub;
use yazi_macro::{log_if_err, succ};
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
				let group = &input.main.history.name;
				let value = input.main.value();
				if input.histories.remember(group, value) {
					log_if_err!(Pubsub::pub_after_history(group, value));
				}
			}
			InputMutGuard::Alt(input, guard) => {
				let group = &guard.history.name;
				let value = guard.value();
				if input.histories.remember(group, value) {
					log_if_err!(Pubsub::pub_after_history(group, value));
				}
			}
		}

		input.history.take();
		succ!();
	}
}
