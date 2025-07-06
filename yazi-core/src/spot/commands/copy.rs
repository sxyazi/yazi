use yazi_parser::spot::CopyOpt;
use yazi_widgets::CLIPBOARD;

use crate::spot::Spot;

impl Spot {
	#[yazi_codegen::command]
	pub fn copy(&mut self, opt: CopyOpt) {
		let Some(lock) = &self.lock else { return };
		let Some(table) = lock.table() else { return };

		let mut s = String::new();
		match opt.r#type.as_ref() {
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
