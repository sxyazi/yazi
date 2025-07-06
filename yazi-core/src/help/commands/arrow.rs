use yazi_adapter::Dimension;
use yazi_macro::render;
use yazi_parser::help::ArrowOpt;
use yazi_widgets::Scrollable;

use crate::help::{HELP_MARGIN, Help};

impl Help {
	#[yazi_codegen::command]
	pub fn arrow(&mut self, opt: ArrowOpt) {
		render!(self.scroll(opt.step));
	}
}

impl Scrollable for Help {
	#[inline]
	fn total(&self) -> usize { self.bindings.len() }

	#[inline]
	fn limit(&self) -> usize { Dimension::available().rows.saturating_sub(HELP_MARGIN) as usize }

	#[inline]
	fn cursor_mut(&mut self) -> &mut usize { &mut self.cursor }

	#[inline]
	fn offset_mut(&mut self) -> &mut usize { &mut self.offset }
}
