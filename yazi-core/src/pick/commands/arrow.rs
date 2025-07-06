use yazi_config::YAZI;
use yazi_macro::render;
use yazi_parser::pick::ArrowOpt;
use yazi_widgets::Scrollable;

use crate::pick::Pick;

impl Pick {
	#[yazi_codegen::command]
	pub fn arrow(&mut self, opt: ArrowOpt) {
		render!(self.scroll(opt.step));
	}
}

impl Scrollable for Pick {
	#[inline]
	fn total(&self) -> usize { self.items.len() }

	#[inline]
	fn limit(&self) -> usize {
		self.position.offset.height.saturating_sub(YAZI.pick.border()) as usize
	}

	#[inline]
	fn cursor_mut(&mut self) -> &mut usize { &mut self.cursor }

	#[inline]
	fn offset_mut(&mut self) -> &mut usize { &mut self.offset }
}
