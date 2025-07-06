use yazi_macro::render;
use yazi_parser::cmp::ArrowOpt;
use yazi_widgets::Scrollable;

use crate::cmp::Cmp;

impl Cmp {
	#[yazi_codegen::command]
	pub fn arrow(&mut self, opt: ArrowOpt) {
		render!(self.scroll(opt.step));
	}
}

impl Scrollable for Cmp {
	#[inline]
	fn total(&self) -> usize { self.cands.len() }

	#[inline]
	fn limit(&self) -> usize { self.cands.len().min(10) }

	#[inline]
	fn cursor_mut(&mut self) -> &mut usize { &mut self.cursor }

	#[inline]
	fn offset_mut(&mut self) -> &mut usize { &mut self.offset }
}
