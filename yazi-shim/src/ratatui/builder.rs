use ansi_to_tui::IntoText;
use ratatui::{text::Text, widgets::Wrap};

use crate::ratatui::LineIter;

pub struct LineIterBuilder {
	tab_size: u8,
	ansi:     bool,
	wrap:     Option<Wrap>,
	width:    u16,
}

impl Default for LineIterBuilder {
	fn default() -> Self { Self { tab_size: 2, ansi: false, wrap: None, width: u16::MAX } }
}

impl LineIterBuilder {
	pub fn tab_size(mut self, size: u8) -> Self {
		self.tab_size = size;
		self
	}

	pub fn ansi(mut self, ansi: bool) -> Self {
		self.ansi = ansi;
		self
	}

	pub fn wrap(mut self, wrap: Option<Wrap>) -> Self {
		self.wrap = wrap;
		self
	}

	pub fn width(mut self, width: u16) -> Self {
		self.width = width;
		self
	}

	pub fn build<'text>(self, s: &'text str) -> Result<LineIter<'text>, ansi_to_tui::Error> {
		let line = if self.ansi {
			LineIter::parsed(parse_ansi_text(s)?.lines, self.tab_size)
		} else {
			LineIter::source(s, self.tab_size)
		};

		Ok(match self.wrap {
			Some(wrap) => line.wrapped(wrap, self.width),
			None => line,
		})
	}
}

fn parse_ansi_text<'text>(s: &'text str) -> Result<Text<'text>, ansi_to_tui::Error> {
	let text = s.to_text()?;
	debug_assert!(
		text.lines.iter().flat_map(|l| l.spans.iter()).all(|span| {
			matches!(span.content, std::borrow::Cow::Borrowed(_))
		}),
		"ansi_to_tui produced Cow::Owned content; the transmute below is unsound"
	);
	// SAFETY: The zero-copy parser creates Spans whose content borrows from the
	// input bytes.  The trait method's `'_` lifetime is tied to the method receiver
	// (`&&'text str`) rather than to the underlying string data (`&'text str`), so
	// we widen it back to `'text` here.  The debug_assert above verifies in debug
	// builds that all Spans indeed contain Cow::Borrowed content.
	unsafe { Ok(std::mem::transmute::<Text<'_>, Text<'text>>(text)) }
}
