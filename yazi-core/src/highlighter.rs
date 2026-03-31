use std::{io::{BufRead, BufReader, Cursor, Seek}, path::PathBuf, sync::OnceLock};

use anyhow::{Result, anyhow, bail};
use ratatui::{layout::Size, text::{Line, Span, Text}};
use syntect::{LoadingError, dumps, easy::HighlightLines, highlighting::{self, Theme, ThemeSet}, parsing::{SyntaxReference, SyntaxSet}};
use yazi_config::{THEME, YAZI};
use yazi_runner::previewer::PeekError;
use yazi_shared::{Id, Ids, replace_to_printable};
use yazi_shim::ratatui::LineIter;

static INCR: Ids = Ids::new();

pub struct Highlighter {
	path:   PathBuf,
	reader: BufReader<std::fs::File>,

	skip:   usize,
	size:   Size,
	ticket: Id,

	theme:    &'static Theme,
	syntaxes: &'static SyntaxSet,
	inner:    Option<HighlightLines<'static>>,
	syntax:   Option<&'static SyntaxReference>,
}

impl Highlighter {
	pub async fn oneshot<P>(path: P, skip: usize, size: Size) -> Result<Text<'static>, PeekError>
	where
		P: Into<PathBuf>,
	{
		let path = path.into();
		tokio::task::spawn_blocking(move || Self::make(path, skip, size)?.highlight()).await?
	}

	fn make<P>(path: P, skip: usize, size: Size) -> Result<Self>
	where
		P: Into<PathBuf>,
	{
		static CACHE: OnceLock<(Theme, SyntaxSet)> = OnceLock::new();

		let path = path.into();
		let (theme, syntaxes) = CACHE.get_or_init(Self::load);

		Ok(Self {
			reader: BufReader::new(std::fs::File::open(&path)?),
			path,

			skip,
			size,
			ticket: INCR.current(),

			theme,
			syntaxes,
			inner: None,
			syntax: None,
		})
	}

	pub fn abort() { INCR.next(); }

	fn highlight(mut self) -> Result<Text<'static>, PeekError> {
		self.load_syntax()?;
		let mut plain = self.syntax.is_none();

		let mut i = 0;
		let mut buf = vec![];
		let mut lines = Vec::with_capacity(self.size.height as usize);
		let mut inspected = 0u16;
		while self.reader.read_until(b'\n', &mut buf).is_ok_and(|n| n > 0) {
			if Self::is_binary(&buf, &mut inspected) {
				Err(anyhow!("Binary file"))?;
			}

			let remaining = Self::normalize_control_chars(&mut buf);
			if remaining || buf.len() > 5000 {
				plain = true;
			}

			self.ensure_not_cancelled()?;
			if plain && !self.process_plain(&buf, &mut i, &mut lines)? {
				break;
			} else if !plain && !self.process_hyper(&buf, &mut i, &mut lines)? {
				break;
			}
			buf.clear();
		}

		if self.skip > 0 && i < self.skip + self.size.height as usize {
			return Err(PeekError::Exceeded(i.saturating_sub(self.size.height as _)));
		}

		Ok(Text::from(lines))
	}

	fn process_plain(&mut self, buf: &[u8], i: &mut usize, lines: &mut Vec<Line>) -> Result<bool> {
		let b = replace_to_printable(buf, true, YAZI.preview.tab_size, false);
		let s = String::from_utf8_lossy(&b);

		let mut it = LineIter::source(&s, YAZI.preview.tab_size);
		if let Some(wrap) = YAZI.preview.wrap.into() {
			it = it.wrapped(wrap, self.size.width);
		}

		while let Some((spans, _)) = it.next() {
			*i += 1;
			if *i > self.skip + self.size.height as usize {
				return Ok(false);
			} else if *i > self.skip {
				lines.push(spans.into_static_line());
			}
			self.ensure_not_cancelled()?;
		}
		Ok(true)
	}

	fn process_hyper(&mut self, buf: &[u8], i: &mut usize, lines: &mut Vec<Line>) -> Result<bool> {
		let Some(syntax) = self.syntax else { bail!("No syntax") };
		let h = self.inner.get_or_insert_with(|| HighlightLines::new(syntax, self.theme));

		let s = String::from_utf8_lossy(buf);
		let line = [Self::to_line_widget(h.highlight_line(&s, self.syntaxes)?)];

		let mut it = LineIter::parsed(&line, YAZI.preview.tab_size);
		if let Some(wrap) = YAZI.preview.wrap.into() {
			it = it.wrapped(wrap, self.size.width);
		}

		while let Some((spans, _)) = it.next() {
			*i += 1;
			if *i > self.skip + self.size.height as usize {
				return Ok(false);
			} else if *i > self.skip {
				lines.push(spans.into_static_line());
			}
			self.ensure_not_cancelled()?;
		}
		Ok(true)
	}

	#[inline]
	fn ensure_not_cancelled(&self) -> Result<(), PeekError> {
		if self.ticket != INCR.current() { Err(anyhow!("Highlighting cancelled"))? } else { Ok(()) }
	}

	fn load() -> (Theme, SyntaxSet) {
		let theme = std::fs::File::open(&THEME.mgr.syntect_theme)
			.map_err(LoadingError::Io)
			.and_then(|f| ThemeSet::load_from_reader(&mut std::io::BufReader::new(f)))
			.or_else(|_| ThemeSet::load_from_reader(&mut Cursor::new(yazi_prebuilt::ansi_theme())));

		let syntaxes = dumps::from_uncompressed_data(yazi_prebuilt::syntaxes());

		(theme.unwrap(), syntaxes.unwrap())
	}

	fn load_syntax(&mut self) -> Result<()> {
		let name = self.path.file_name().map(|n| n.to_string_lossy()).unwrap_or_default();
		if let Some(s) = self.syntaxes.find_syntax_by_extension(&name) {
			self.syntax = Some(s);
			return Ok(());
		}

		let ext = self.path.extension().map(|e| e.to_string_lossy()).unwrap_or_default();
		if let Some(s) = self.syntaxes.find_syntax_by_extension(&ext) {
			self.syntax = Some(s);
			return Ok(());
		}

		let mut buf = vec![];
		if self.reader.read_until(b'\n', &mut buf).is_ok_and(|n| n > 0) {
			self.reader.rewind()?;
			self.syntax = self.syntaxes.find_syntax_by_first_line(&String::from_utf8_lossy(&buf));
		}
		Ok(())
	}

	#[inline(always)]
	fn is_binary(buf: &[u8], inspected: &mut u16) -> bool {
		if let Some(n) = 1024u16.checked_sub(*inspected) {
			*inspected += n.min(buf.len() as u16);
			buf.iter().take(n as usize).any(|&b| b == 0)
		} else {
			false
		}
	}

	fn normalize_control_chars(buf: &mut Vec<u8>) -> bool {
		if buf.ends_with(b"\r\n") {
			buf.pop();
			buf.pop();
			buf.push(b'\n');
		}

		let mut remaining = false;
		for b in buf.iter_mut() {
			if *b == b'\r' {
				*b = b'\n';
			} else {
				remaining |= matches!(b, 0..=0x08 | 0x0B..=0x1F | 0x7F);
			}
		}
		remaining
	}
}

impl Highlighter {
	fn to_line_widget<'a>(regions: Vec<(highlighting::Style, &'a str)>) -> Line<'a> {
		Line::from_iter(regions.into_iter().map(|(style, s)| {
			let mut modifier = ratatui::style::Modifier::empty();
			if style.font_style.contains(highlighting::FontStyle::BOLD) {
				modifier |= ratatui::style::Modifier::BOLD;
			}
			if style.font_style.contains(highlighting::FontStyle::ITALIC) {
				modifier |= ratatui::style::Modifier::ITALIC;
			}
			if style.font_style.contains(highlighting::FontStyle::UNDERLINE) {
				modifier |= ratatui::style::Modifier::UNDERLINED;
			}

			Span {
				content: s.into(),
				style:   ratatui::style::Style {
					fg: Self::to_ansi_color(style.foreground),
					// bg: Self::to_ansi_color(style.background),
					add_modifier: modifier,
					..Default::default()
				},
			}
		}))
	}

	// Copied from https://github.com/sharkdp/bat/blob/master/src/terminal.rs
	fn to_ansi_color(color: highlighting::Color) -> Option<ratatui::style::Color> {
		if color.a == 0 {
			// Themes can specify one of the user-configurable terminal colors by
			// encoding them as #RRGGBBAA with AA set to 00 (transparent) and RR set
			// to the 8-bit color palette number. The built-in themes ansi, base16,
			// and base16-256 use this.
			Some(match color.r {
				// For the first 8 colors, use the Color enum to produce ANSI escape
				// sequences using codes 30-37 (foreground) and 40-47 (background).
				// For example, red foreground is \x1b[31m. This works on terminals
				// without 256-color support.
				0x00 => ratatui::style::Color::Black,
				0x01 => ratatui::style::Color::Red,
				0x02 => ratatui::style::Color::Green,
				0x03 => ratatui::style::Color::Yellow,
				0x04 => ratatui::style::Color::Blue,
				0x05 => ratatui::style::Color::Magenta,
				0x06 => ratatui::style::Color::Cyan,
				0x07 => ratatui::style::Color::White,
				// For all other colors, use Fixed to produce escape sequences using
				// codes 38;5 (foreground) and 48;5 (background). For example,
				// bright red foreground is \x1b[38;5;9m. This only works on
				// terminals with 256-color support.
				//
				// TODO: When ansi_term adds support for bright variants using codes
				// 90-97 (foreground) and 100-107 (background), we should use those
				// for values 0x08 to 0x0f and only use Fixed for 0x10 to 0xff.
				n => ratatui::style::Color::Indexed(n),
			})
		} else if color.a == 1 {
			// Themes can specify the terminal's default foreground/background color
			// (i.e. no escape sequence) using the encoding #RRGGBBAA with AA set to
			// 01. The built-in theme ansi uses this.
			None
		} else {
			Some(ratatui::style::Color::Rgb(color.r, color.g, color.b))
		}
	}
}
