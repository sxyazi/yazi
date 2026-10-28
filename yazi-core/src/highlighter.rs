use std::{io::{BufRead, BufReader, Cursor, Seek}, path::{Path, PathBuf}, sync::{Arc, LazyLock}};

use anyhow::{Result, anyhow};
use parking_lot::Mutex;
use ratatui_core::{layout::Size, text::{Line, Span, Text}};
use syntect::{LoadingError, dumps, easy::HighlightLines, highlighting::{self, Theme, ThemeSet}, parsing::{SyntaxReference, SyntaxSet}};
use yazi_config::{THEME, YAZI};
use yazi_runner::previewer::PeekError;
use yazi_shared::{replace_to_printable, scope::Scope};
use yazi_shim::ratatui::LineIter;

static SYNTAXES: LazyLock<SyntaxSet> =
	LazyLock::new(|| dumps::from_uncompressed_data(yazi_prebuilt::syntaxes()).unwrap());

pub struct Highlighter {
	reader: BufReader<std::fs::File>,

	skip:  usize,
	size:  Size,
	scope: Scope,

	theme:  Arc<Theme>,
	syntax: Option<&'static SyntaxReference>,
}

impl Highlighter {
	pub async fn oneshot<P>(
		path: P,
		skip: usize,
		size: Size,
		scope: Scope,
	) -> Result<Text<'static>, PeekError>
	where
		P: Into<PathBuf>,
	{
		let path = path.into();
		tokio::task::spawn_blocking(move || Self::make(path, skip, size, scope)?.highlight()).await?
	}

	fn make(path: PathBuf, skip: usize, size: Size, scope: Scope) -> Result<Self> {
		let mut reader = BufReader::new(std::fs::File::open(&path)?);
		let theme = Self::load_theme();
		let syntax = Self::load_syntax(&path, &mut reader)?;

		Ok(Self { reader, skip, size, scope, theme, syntax })
	}

	fn highlight(mut self) -> Result<Text<'static>, PeekError> {
		let mut h = self.syntax.map(|syntax| HighlightLines::new(syntax, &self.theme));

		let mut i = 0;
		let mut buf = vec![];
		let mut lines = Vec::with_capacity(self.size.height as usize);
		let mut inspected = 0;
		while self.reader.read_until(b'\n', &mut buf).is_ok_and(|n| n > 0) {
			if Self::is_binary(&buf, &mut inspected) {
				Err(anyhow!("Binary file"))?;
			}

			if Self::normalize_control_chars(&mut buf) || buf.len() > 5000 {
				h = None;
			}

			self.ensure_not_cancelled()?;
			if !self.process(&buf, &mut i, &mut lines, h.as_mut())? {
				break;
			}

			buf.clear();
		}

		if self.skip > 0 && i < self.skip + self.size.height as usize {
			return Err(PeekError::Exceeded(i.saturating_sub(self.size.height as _)));
		}

		Ok(Text::from(lines))
	}

	fn process(
		&self,
		buf: &[u8],
		i: &mut usize,
		lines: &mut Vec<Line>,
		h: Option<&mut HighlightLines<'_>>,
	) -> Result<bool, PeekError> {
		let normalized;
		let text;
		let line;
		let mut it = if let Some(h) = h {
			text = String::from_utf8_lossy(buf);
			line = [Self::to_line_widget(h.highlight_line(&text, &SYNTAXES)?)];
			LineIter::parsed(&line, YAZI.preview.tab_size)
		} else {
			normalized = replace_to_printable(buf, true, YAZI.preview.tab_size, false);
			text = String::from_utf8_lossy(&normalized);
			LineIter::source(&text, YAZI.preview.tab_size)
		};

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
		if self.scope.is_cancelled() { Err(PeekError::Cancelled) } else { Ok(()) }
	}

	fn load_theme() -> Arc<Theme> {
		static CACHE: Mutex<Option<(Arc<PathBuf>, Arc<Theme>)>> = Mutex::new(None);

		let path = &*THEME.mgr.syntect_theme.load();
		let load = || {
			let theme = std::fs::File::open(path.as_path())
				.map_err(LoadingError::Io)
				.and_then(|f| ThemeSet::load_from_reader(&mut std::io::BufReader::new(f)))
				.or_else(|_| ThemeSet::load_from_reader(&mut Cursor::new(yazi_prebuilt::ansi_theme())))
				.unwrap();
			(path.clone(), Arc::new(theme))
		};

		match &mut *CACHE.lock() {
			Some((p, theme)) if p == path => theme.clone(),
			slot => slot.insert(load()).1.clone(),
		}
	}

	fn load_syntax(
		path: &Path,
		reader: &mut BufReader<std::fs::File>,
	) -> Result<Option<&'static SyntaxReference>> {
		let name = path.file_name().map(|n| n.to_string_lossy()).unwrap_or_default();
		if let Some(s) = SYNTAXES.find_syntax_by_extension(&name) {
			return Ok(Some(s));
		}

		let ext = path.extension().map(|e| e.to_string_lossy()).unwrap_or_default();
		if let Some(s) = SYNTAXES.find_syntax_by_extension(&ext) {
			return Ok(Some(s));
		}

		let mut buf = vec![];
		if reader.read_until(b'\n', &mut buf).is_ok_and(|n| n > 0) {
			reader.rewind()?;
			Ok(SYNTAXES.find_syntax_by_first_line(&String::from_utf8_lossy(&buf)))
		} else {
			Ok(None)
		}
	}

	#[inline(always)]
	fn is_binary(buf: &[u8], inspected: &mut usize) -> bool {
		let n = buf.len().min(1024 - *inspected);
		*inspected += n;
		buf[..n].contains(&0)
	}

	fn normalize_control_chars(buf: &mut Vec<u8>) -> bool {
		if buf.ends_with(b"\r\n") {
			buf.remove(buf.len() - 2);
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
			let mut modifier = ratatui_core::style::Modifier::empty();
			if style.font_style.contains(highlighting::FontStyle::BOLD) {
				modifier |= ratatui_core::style::Modifier::BOLD;
			}
			if style.font_style.contains(highlighting::FontStyle::ITALIC) {
				modifier |= ratatui_core::style::Modifier::ITALIC;
			}
			if style.font_style.contains(highlighting::FontStyle::UNDERLINE) {
				modifier |= ratatui_core::style::Modifier::UNDERLINED;
			}

			Span {
				content: s.into(),
				style:   ratatui_core::style::Style {
					fg: Self::to_ansi_color(style.foreground),
					// bg: Self::to_ansi_color(style.background),
					add_modifier: modifier,
					..Default::default()
				},
			}
		}))
	}

	// Copied from https://github.com/sharkdp/bat/blob/master/src/terminal.rs
	fn to_ansi_color(color: highlighting::Color) -> Option<ratatui_core::style::Color> {
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
				0x00 => ratatui_core::style::Color::Black,
				0x01 => ratatui_core::style::Color::Red,
				0x02 => ratatui_core::style::Color::Green,
				0x03 => ratatui_core::style::Color::Yellow,
				0x04 => ratatui_core::style::Color::Blue,
				0x05 => ratatui_core::style::Color::Magenta,
				0x06 => ratatui_core::style::Color::Cyan,
				0x07 => ratatui_core::style::Color::White,
				// For all other colors, use Fixed to produce escape sequences using
				// codes 38;5 (foreground) and 48;5 (background). For example,
				// bright red foreground is \x1b[38;5;9m. This only works on
				// terminals with 256-color support.
				//
				// TODO: When ansi_term adds support for bright variants using codes
				// 90-97 (foreground) and 100-107 (background), we should use those
				// for values 0x08 to 0x0f and only use Fixed for 0x10 to 0xff.
				n => ratatui_core::style::Color::Indexed(n),
			})
		} else if color.a == 1 {
			// Themes can specify the terminal's default foreground/background color
			// (i.e. no escape sequence) using the encoding #RRGGBBAA with AA set to
			// 01. The built-in theme ansi uses this.
			None
		} else {
			Some(ratatui_core::style::Color::Rgb(color.r, color.g, color.b))
		}
	}
}
