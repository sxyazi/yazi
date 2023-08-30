use config::{keymap::{Control, KeymapLayer}, KEYMAP};
use shared::Term;

use super::HELP_MARGIN;
use crate::emit;

#[derive(Default)]
pub struct Help {
	pub visible: bool,
	bindings:    Vec<Control>,

	offset: usize,
	cursor: usize,
}

impl Help {
	#[inline]
	pub fn limit() -> usize { Term::size().rows.saturating_sub(HELP_MARGIN) as usize }

	pub fn toggle(&mut self, layer: KeymapLayer) -> bool {
		self.visible = !self.visible;
		self.bindings = if self.visible { KEYMAP.get(layer).clone() } else { Vec::new() };

		emit!(Peek); // Show/hide preview for images
		true
	}

	pub fn escape(&mut self) -> bool { todo!() }

	#[inline]
	pub fn arrow(&mut self, step: isize) -> bool {
		if step > 0 { self.next(step as usize) } else { self.prev(step.unsigned_abs()) }
	}

	pub fn next(&mut self, step: usize) -> bool {
		let len = self.bindings.len();
		if len == 0 {
			return false;
		}

		let old = self.cursor;
		self.cursor = (self.cursor + step).min(len - 1);

		let limit = Self::limit();
		if self.cursor >= (self.offset + limit).min(len).saturating_sub(5) {
			self.offset = len.saturating_sub(limit).min(self.offset + self.cursor - old);
		}

		old != self.cursor
	}

	pub fn prev(&mut self, step: usize) -> bool {
		let old = self.cursor;
		self.cursor = self.cursor.saturating_sub(step);

		if self.cursor < self.offset + 5 {
			self.offset = self.offset.saturating_sub(old - self.cursor);
		}

		old != self.cursor
	}

	#[inline]
	pub fn window(&self) -> &[Control] {
		let end = (self.offset + Self::limit()).min(self.bindings.len());
		&self.bindings[self.offset..end]
	}

	pub fn filter(&mut self) -> bool { todo!() }
}

impl Help {
	// --- Cursor
	#[inline]
	pub fn rel_cursor(&self) -> usize { self.cursor - self.offset }
}
