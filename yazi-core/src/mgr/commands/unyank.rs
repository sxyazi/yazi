use yazi_macro::render;
use yazi_parser::mgr::UnyankOpt;

use crate::mgr::Mgr;

impl Mgr {
	#[yazi_codegen::command]
	pub fn unyank(&mut self, _: UnyankOpt) {
		let repeek = self.hovered().is_some_and(|f| f.is_dir() && self.yanked.contains_in(&f.url));
		self.yanked.clear();

		render!(self.yanked.catchup_revision(false));
		if repeek {
			self.peek(true);
		}
	}
}
