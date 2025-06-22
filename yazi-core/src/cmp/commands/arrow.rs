use yazi_fs::Step;
use yazi_macro::render;
use yazi_shared::event::CmdCow;

use crate::{Scrollable, cmp::Cmp};

struct Opt {
	step: Step,
}

impl From<CmdCow> for Opt {
	fn from(c: CmdCow) -> Self {
		Self { step: c.first().and_then(|d| d.try_into().ok()).unwrap_or_default() }
	}
}

impl Cmp {
	#[yazi_codegen::command]
	pub fn arrow(&mut self, opt: Opt) {
		render!(self.scroll(opt.step));
	}
}

impl Scrollable for Cmp {
	#[inline]
	fn len(&self) -> usize { self.cands.len() }

	#[inline]
	fn limit(&self) -> usize { self.cands.len().min(10) }

	#[inline]
	fn cursor_mut(&mut self) -> &mut usize { &mut self.cursor }

	#[inline]
	fn offset_mut(&mut self) -> &mut usize { &mut self.offset }
}
