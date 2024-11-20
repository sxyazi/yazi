use yazi_plugin::CLIPBOARD;
use yazi_shared::event::Cmd;

use crate::spot::Spot;

struct Opt {
	type_: String,
}

impl From<Cmd> for Opt {
	fn from(mut c: Cmd) -> Self { Self { type_: c.take_first_str().unwrap_or_default() } }
}

impl Spot {
	#[yazi_codegen::command]
	pub fn copy(&mut self, opt: Opt) {
		let Some(lock) = &self.lock else { return };
		let Some(table) = lock.table() else { return };

		let mut s = String::new();
		match opt.type_.as_str() {
			"cell" => {
				let Some(cell) = table.selected_cell() else { return };
				s = cell.to_string();
			}
			"line" => {
				// TODO
			}
			_ => {}
		}

		futures::executor::block_on(CLIPBOARD.set(s));
	}
}
