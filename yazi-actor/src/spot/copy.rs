use anyhow::Result;
use yazi_macro::succ;
use yazi_parser::spot::CopyForm;
use yazi_shared::data::Data;
use yazi_widgets::CLIPBOARD;

use crate::{Actor, Ctx};

pub struct Copy;

impl Actor for Copy {
	type Form = CopyForm;

	const NAME: &str = "copy";

	fn act(cx: &mut Ctx, form: Self::Form) -> Result<Data> {
		let spot = &cx.tab().spot;
		let Some(lock) = &spot.lock else { succ!() };
		let Some(table) = lock.table() else { succ!() };

		let mut s = String::new();
		match form.r#type.as_ref() {
			"cell" => {
				let Some(cell) = table.selected_cell() else { succ!() };
				s = cell.to_string();
			}
			"line" => {
				// TODO
			}
			_ => {}
		}

		futures::executor::block_on(CLIPBOARD.set(s));
		succ!();
	}
}
