use yazi_config::YAZI;
use yazi_macro::render;
use yazi_shared::event::CmdCow;
use yazi_widgets::{Scrollable, Step};

use crate::pick::Pick;

struct Opt {
	step: Step,
}

impl From<CmdCow> for Opt {
	fn from(c: CmdCow) -> Self {
		Self { step: c.first().and_then(|d| d.try_into().ok()).unwrap_or_default() }
	}
}

impl Pick {
	#[yazi_codegen::command]
	pub fn arrow(&mut self, opt: Opt) {
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
