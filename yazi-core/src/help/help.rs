use crossterm::event::KeyCode;
use unicode_width::UnicodeWidthStr;
use yazi_config::{keymap::{Control, Key, KeymapLayer}, KEYMAP};
use yazi_shared::Term;

use super::HELP_MARGIN;
use crate::input::Input;

#[derive(Default)]
pub struct Help {
	pub visible:         bool,
	pub layer:           KeymapLayer,
	pub(super) bindings: Vec<Control>,

	// Filter
	keyword:              Option<String>,
	pub(super) in_filter: Option<Input>,

	pub(super) offset: usize,
	pub(super) cursor: usize,
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

		// TODO: Peek
		// emit!(Peek); // Show/hide preview for images
		true
	}

	pub(super) fn filter_apply(&mut self) -> bool {
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

		let b = match &key {
			Key { code: KeyCode::Backspace, shift: false, ctrl: false, alt: false } => {
				input.backspace(false)
			}
			_ => input.type_(key),
		};

		if b { self.filter_apply() } else { false }
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
