use yazi_config::{keymap::{Control, Key, KeymapLayer}, KEYMAP};
use yazi_shared::Term;
use unicode_width::UnicodeWidthStr;

use super::HELP_MARGIN;
use crate::{emit, input::Input};

#[derive(Default)]
pub struct Help {
	pub visible: bool,
	pub layer:   KeymapLayer,
	bindings:    Vec<Control>,

	// Filter
	keyword:   Option<String>,
	in_filter: Option<Input>,

	offset: usize,
	cursor: usize,
}

impl Help {
	#[inline]
	pub fn limit() -> usize { Term::size().rows.saturating_sub(HELP_MARGIN) as usize }

	pub fn toggle(&mut self, layer: KeymapLayer) -> bool {
		self.visible = !self.visible;
		self.layer = layer;

		self.keyword = Some(String::new());
		self.in_filter = None;
		self.filter_apply();

		self.offset = 0;
		self.cursor = 0;

		emit!(Peek); // Show/hide preview for images
		true
	}

	pub fn escape(&mut self) -> bool {
		if self.in_filter.is_some() {
			self.in_filter = None;
			self.filter_apply();
			true
		} else {
			self.toggle(self.layer)
		}
	}

	#[inline]
	pub fn arrow(&mut self, step: isize) -> bool {
		let max = self.bindings.len().saturating_sub(1);
		self.offset = self.offset.min(max);
		self.cursor = self.cursor.min(max);

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

	pub fn filter(&mut self) -> bool {
		self.in_filter = Some(Default::default());
		self.filter_apply();
		true
	}

	fn filter_apply(&mut self) -> bool {
		let kw = self.in_filter.as_ref().map(|i| i.value()).filter(|v| !v.is_empty());
		if self.keyword.as_deref() == kw {
			return false;
		}

		if let Some(kw) = kw {
			self.bindings = KEYMAP.get(self.layer).iter().filter(|&c| c.contains(kw)).cloned().collect();
		} else {
			self.bindings = KEYMAP.get(self.layer).clone();
		}

		self.keyword = kw.map(|s| s.to_owned());
		self.arrow(0);
		true
	}

	pub fn type_(&mut self, key: &Key) -> bool {
		let Some(input) = &mut self.in_filter else {
			return false;
		};

		if key.is_enter() {
			self.in_filter = None;
			return true;
		}

		if input.type_(key) {
			return self.filter_apply();
		}

		false
	}
}

impl Help {
	// --- Keyword
	#[inline]
	pub fn keyword(&self) -> Option<String> {
		self
			.in_filter
			.as_ref()
			.map(|i| i.value())
			.or(self.keyword.as_deref())
			.map(|s| format!("/{}", s))
	}

	// --- Bindings
	#[inline]
	pub fn window(&self) -> &[Control] {
		let end = (self.offset + Self::limit()).min(self.bindings.len());
		&self.bindings[self.offset..end]
	}

	// --- Cursor
	#[inline]
	pub fn cursor(&self) -> Option<(u16, u16)> {
		if !self.visible || self.in_filter.is_none() {
			return None;
		}
		if let Some(kw) = self.keyword() {
			return Some((kw.width() as u16, Term::size().rows));
		}
		None
	}

	#[inline]
	pub fn rel_cursor(&self) -> usize { self.cursor - self.offset }
}
