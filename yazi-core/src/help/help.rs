use crossterm::{cursor::SetCursorStyle, event::KeyCode};
use unicode_width::UnicodeWidthStr;
use yazi_adapter::Dimension;
use yazi_config::{KEYMAP, YAZI, keymap::{Chord, Key}};
use yazi_macro::{render, render_and};
use yazi_shared::Layer;

use crate::Scrollable;

#[derive(Default)]
pub struct Help {
	pub visible:         bool,
	pub layer:           Layer,
	pub(super) bindings: Vec<&'static Chord>,

	// Filter
	pub(super) keyword:   String,
	pub(super) in_filter: Option<yazi_widgets::input::Input>,

	pub(super) offset: usize,
	pub(super) cursor: usize,
}

impl Help {
	pub fn toggle(&mut self, layer: Layer) {
		self.visible = !self.visible;
		self.layer = layer;

		self.keyword = String::new();
		self.in_filter = None;
		self.filter_apply();

		self.offset = 0;
		self.cursor = 0;
		render!();
	}

	pub fn r#type(&mut self, key: &Key) -> bool {
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
				input.r#type(key);
			}
		}

		self.filter_apply();
		true
	}

	pub(super) fn filter_apply(&mut self) {
		let kw = self.in_filter.as_ref().map_or("", |i| i.value());

		if kw.is_empty() {
			self.keyword = String::new();
			self.bindings = KEYMAP.get(self.layer).iter().collect();
		} else if self.keyword != kw {
			self.keyword = kw.to_owned();
			self.bindings = KEYMAP.get(self.layer).iter().filter(|&c| c.contains(kw)).collect();
		}

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
			.or(Some(self.keyword.as_str()).filter(|&s| !s.is_empty()))
			.map(|s| format!("Filter: {s}"))
	}

	// --- Bindings
	#[inline]
	pub fn window(&self) -> &[&Chord] {
		let end = (self.offset + self.limit()).min(self.bindings.len());
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

	#[inline]
	pub fn cursor_shape(&self) -> SetCursorStyle {
		if YAZI.input.cursor_blink {
			SetCursorStyle::BlinkingBlock
		} else {
			SetCursorStyle::SteadyBlock
		}
	}
}
