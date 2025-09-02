use anyhow::Result;
use crossterm::{cursor::SetCursorStyle, event::KeyCode};
use unicode_width::UnicodeWidthStr;
use yazi_adapter::Dimension;
use yazi_config::{KEYMAP, YAZI, keymap::{Chord, Key}};
use yazi_macro::{act, render, render_and};
use yazi_shared::Layer;
use yazi_widgets::Scrollable;

use crate::help::HELP_MARGIN;

#[derive(Default)]
pub struct Help {
	pub visible:         bool,
	pub layer:           Layer,
	pub(super) bindings: Vec<&'static Chord>,

	// Filter
	pub keyword:   String,
	pub in_filter: Option<yazi_widgets::input::Input>,

	pub offset: usize,
	pub cursor: usize,
}

impl Help {
	pub fn r#type(&mut self, key: &Key) -> Result<bool> {
		let Some(input) = &mut self.in_filter else { return Ok(false) };
		match key {
			Key { code: KeyCode::Esc, shift: false, ctrl: false, alt: false, super_: false } => {
				self.in_filter = None;
				render!();
			}
			Key { code: KeyCode::Enter, shift: false, ctrl: false, alt: false, super_: false } => {
				self.in_filter = None;
				return Ok(render_and!(true)); // Don't do the `filter_apply` below, since we already have the filtered results.
			}
			Key { code: KeyCode::Backspace, shift: false, ctrl: false, alt: false, super_: false } => {
				act!(backspace, input)?;
			}
			_ => {
				input.r#type(key)?;
			}
		}

		self.filter_apply();
		Ok(true)
	}

	pub fn filter_apply(&mut self) {
		let kw = self.in_filter.as_ref().map_or("", |i| i.value());

		if kw.is_empty() {
			self.keyword = String::new();
			self.bindings = KEYMAP.get(self.layer).iter().collect();
		} else if self.keyword != kw {
			self.keyword = kw.to_owned();
			self.bindings = KEYMAP.get(self.layer).iter().filter(|&c| c.contains(kw)).collect();
		}

		render!(self.scroll(0));
	}
}

impl Help {
	// --- Keyword
	pub fn keyword(&self) -> Option<String> {
		self
			.in_filter
			.as_ref()
			.map(|i| i.value())
			.or(Some(self.keyword.as_str()).filter(|&s| !s.is_empty()))
			.map(|s| format!("Filter: {s}"))
	}

	// --- Bindings
	pub fn window(&self) -> &[&Chord] {
		let end = (self.offset + self.limit()).min(self.bindings.len());
		&self.bindings[self.offset..end]
	}

	// --- Cursor
	pub fn cursor(&self) -> Option<(u16, u16)> {
		if !self.visible || self.in_filter.is_none() {
			return None;
		}
		if let Some(kw) = self.keyword() {
			return Some((kw.width() as u16, Dimension::available().rows));
		}
		None
	}

	pub fn rel_cursor(&self) -> usize { self.cursor - self.offset }

	pub fn cursor_shape(&self) -> SetCursorStyle {
		if YAZI.input.cursor_blink {
			SetCursorStyle::BlinkingBlock
		} else {
			SetCursorStyle::SteadyBlock
		}
	}
}

impl Scrollable for Help {
	fn total(&self) -> usize { self.bindings.len() }

	fn limit(&self) -> usize { Dimension::available().rows.saturating_sub(HELP_MARGIN) as usize }

	fn cursor_mut(&mut self) -> &mut usize { &mut self.cursor }

	fn offset_mut(&mut self) -> &mut usize { &mut self.offset }
}
