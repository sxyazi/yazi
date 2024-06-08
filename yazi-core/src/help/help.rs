use crossterm::event::KeyCode;
use unicode_width::UnicodeWidthStr;
use yazi_adapter::Dimension;
use yazi_config::{keymap::{Control, Key}, KEYMAP};
use yazi_shared::{render, render_and, Layer};

use super::HELP_MARGIN;
use crate::input::Input;

#[derive(Default)]
pub struct Help {
	pub visible:         bool,
	pub layer:           Layer,
	pub(super) bindings: Vec<&'static Control>,

	// Filter
	keyword:              Option<String>,
	pub(super) in_filter: Option<Input>,

	pub(super) offset: usize,
	pub(super) cursor: usize,
}

impl Help {
	#[inline]
	pub fn limit() -> usize { Dimension::available().rows.saturating_sub(HELP_MARGIN) as usize }

	pub fn toggle(&mut self, layer: Layer) {
		self.visible = !self.visible;
		self.layer = layer;

		self.keyword = Some(String::new());
		self.in_filter = None;
		self.filter_apply();

		self.offset = 0;
		self.cursor = 0;
		render!();
	}

	pub fn type_(&mut self, key: &Key) -> bool {
		let Some(input) = &mut self.in_filter else {
			return false;
		};

		match key {
			Key { code: KeyCode::Esc, shift: false, ctrl: false, alt: false, super_: false } => {
				self.in_filter = None;
				render!();
			}
			Key { code: KeyCode::Enter, shift: false, ctrl: false, alt: false, super_: false } => {
				self.in_filter = None;
				return render_and!(true); // Don't do the `filter_apply` below, since we already have the filtered results.
			}
			Key { code: KeyCode::Backspace, shift: false, ctrl: false, alt: false, super_: false } => {
				input.backspace(false);
			}
			_ => {
				input.type_(key);
			}
		}

		self.filter_apply();
		true
	}

	pub(super) fn filter_apply(&mut self) {
		let kw = self.in_filter.as_ref().map(|i| i.value()).filter(|v| !v.is_empty());
		if self.keyword.as_deref() == kw {
			return;
		}

		if let Some(kw) = kw {
			self.bindings = KEYMAP.get(self.layer).iter().filter(|&c| c.contains(kw)).collect();
		} else {
			self.bindings = KEYMAP.get(self.layer).iter().collect();
		}

		self.keyword = kw.map(|s| s.to_owned());
		self.arrow(0);
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
	pub fn window(&self) -> &[&Control] {
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
			return Some((kw.width() as u16, Dimension::available().rows));
		}
		None
	}

	#[inline]
	pub fn rel_cursor(&self) -> usize { self.cursor - self.offset }
}
