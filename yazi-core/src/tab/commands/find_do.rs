use yazi_macro::render;

use super::find::Opt;
use crate::tab::{Finder, Tab};

impl Tab {
	#[yazi_codegen::command]
	pub fn find_do(&mut self, opt: Opt) {
		let Some(query) = opt.query else {
			return;
		};
		if query.is_empty() {
			self.escape_find();
			return;
		}

		let Ok(finder) = Finder::new(&query, opt.case) else {
			return;
		};
		if matches!(&self.finder, Some(f) if f.filter == finder.filter) {
			return;
		}

		let step = if opt.prev {
			finder.prev(&self.current.files, self.current.cursor, true)
		} else {
			finder.next(&self.current.files, self.current.cursor, true)
		};

		if let Some(step) = step {
			self.arrow(step);
		}

		self.finder = Some(finder);
		render!();
	}
}
