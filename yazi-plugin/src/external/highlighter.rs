use std::{borrow::Cow, io::Cursor, mem, path::{Path, PathBuf}, sync::OnceLock};

use anyhow::{Result, anyhow};
use ratatui::{layout::Size, text::{Line, Span, Text}};
use syntect::{LoadingError, dumps, easy::HighlightLines, highlighting::{self, Theme, ThemeSet}, parsing::{SyntaxReference, SyntaxSet}};
use tokio::{fs::File, io::{AsyncBufReadExt, BufReader}};
use yazi_config::{THEME, YAZI, preview::PreviewWrap};
use yazi_shared::{Ids, errors::PeekError, replace_to_printable};

static INCR: Ids = Ids::new();
static SYNTECT: OnceLock<(Theme, SyntaxSet)> = OnceLock::new();

pub struct Highlighter {
	path: PathBuf,
}

impl Highlighter {
	#[inline]
	pub fn new(path: &Path) -> Self { Self { path: path.to_owned() } }

	pub fn init() -> (&'static Theme, &'static SyntaxSet) {
		let f = || {
			let theme = std::fs::File::open(&THEME.mgr.syntect_theme)
				.map_err(LoadingError::Io)
				.and_then(|f| ThemeSet::load_from_reader(&mut std::io::BufReader::new(f)))
				.or_else(|_| ThemeSet::load_from_reader(&mut Cursor::new(yazi_prebuilt::ansi_theme())));

			let syntaxes = dumps::from_uncompressed_data(yazi_prebuilt::syntaxes());

			(theme.unwrap(), syntaxes.unwrap())
		};

		let (theme, syntaxes) = SYNTECT.get_or_init(f);
		(theme, syntaxes)
	}

	#[inline]
	pub fn abort() { INCR.next(); }

	pub async fn highlight(&self, skip: usize, size: Size) -> Result<Text<'static>, PeekError> {
		let mut reader = BufReader::new(File::open(&self.path).await?);

		let syntax = Self::find_syntax(&self.path).await;
		let mut plain = syntax.is_err();

		let mut before = Vec::with_capacity(if plain { 0 } else { skip });
		let mut after = Vec::with_capacity(size.height as _);

		let mut i = 0;
		let mut buf = vec![];
		let mut inspected = 0u16;
		while reader.read_until(b'\n', &mut buf).await.is_ok_and(|n| n > 0) {
			if Self::is_binary(&buf, &mut inspected) {
				return Err("Binary file".into());
			}

			if !plain && (buf.len() > 5000 || Self::contains_control_chars(&buf)) {
				plain = true;
				drop(mem::take(&mut before));
			}

			if buf.ends_with(b"\r\n") {
				buf.pop();
				buf.pop();
				buf.push(b'\n');
			}

			i += if i >= skip {
				buf.iter_mut().for_each(Self::carriage_return_to_line_feed);
				after.push(String::from_utf8_lossy(&buf).into_owned());
				Self::line_height(&after[after.len() - 1], size.width)
			} else if !plain {
				before.push(String::from_utf8_lossy(&buf).into_owned());
				Self::line_height(&before[before.len() - 1], size.width)
			} else if YAZI.preview.wrap == PreviewWrap::Yes {
				Self::line_height(&String::from_utf8_lossy(&buf), size.width)
			} else {
				1
			};

			buf.clear();
			if i > skip + size.height as usize {
				break;
			}
		}

		if skip > 0 && i < skip + size.height as usize {
			return Err(PeekError::Exceed(i.saturating_sub(size.height as _)));
		}

		Ok(if plain {
			Text::from(replace_to_printable(&after, YAZI.preview.tab_size))
		} else {
			Self::highlight_with(before, after, syntax.unwrap()).await?
		})
	}

	async fn highlight_with(
		before: Vec<String>,
		after: Vec<String>,
		syntax: &'static SyntaxReference,
	) -> Result<Text<'static>, PeekError> {
		let ticket = INCR.current();

		tokio::task::spawn_blocking(move || {
			let (theme, syntaxes) = Self::init();
			let mut h = HighlightLines::new(syntax, theme);

			for line in before {
				if ticket != INCR.current() {
					return Err("Highlighting cancelled".into());
				}
				h.highlight_line(&line, syntaxes).map_err(|e| anyhow!(e))?;
			}

			let indent = YAZI.preview.indent();
			let mut lines = Vec::with_capacity(after.len());
			for line in after {
				if ticket != INCR.current() {
					return Err("Highlighting cancelled".into());
				}

				let regions = h.highlight_line(&line, syntaxes).map_err(|e| anyhow!(e))?;
				lines.push(Self::to_line_widget(regions, &indent));
			}

			Ok(Text::from(lines))
		})
		.await?
	}

	async fn find_syntax(path: &Path) -> Result<&'static SyntaxReference> {
		let (_, syntaxes) = Self::init();
		let name = path.file_name().map(|n| n.to_string_lossy()).unwrap_or_default();
		if let Some(s) = syntaxes.find_syntax_by_extension(&name) {
			return Ok(s);
		}

		let ext = path.extension().map(|e| e.to_string_lossy()).unwrap_or_default();
		if let Some(s) = syntaxes.find_syntax_by_extension(&ext) {
			return Ok(s);
		}

		let mut line = String::new();
		let mut reader = BufReader::new(File::open(&path).await?);
		reader.read_line(&mut line).await?;
		syntaxes.find_syntax_by_first_line(&line).ok_or_else(|| anyhow!("No syntax found"))
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

	fn line_height(s: &str, width: u16) -> usize {
		if YAZI.preview.wrap != PreviewWrap::Yes {
			return 1;
		}

		let pad = YAZI
			.preview
			.tab_size
			.checked_sub(1)
			.map(|n| s.bytes().filter(|&b| b == b'\t').count() * n as usize)
			.map(|n| yazi_config::preview::Preview::indent_with(n))
			.unwrap_or_default();

		let line = Line {
			spans: vec![Span { content: pad, style: Default::default() }, Span {
				content: Cow::Borrowed(s),
				..Default::default()
			}],
			..Default::default()
		};

		ratatui::widgets::Paragraph::new(line)
			.wrap(ratatui::widgets::Wrap { trim: false })
			.line_count(width)
	}

	#[inline(always)]
	fn contains_control_chars(buf: &[u8]) -> bool {
		buf.iter().any(|&b| b.is_ascii_control() && !matches!(b, b'\t' | b'\n' | b'\r'))
	}

	#[inline(always)]
	fn carriage_return_to_line_feed(b: &mut u8) {
		if *b == b'\r' {
			*b = b'\n';
		}
	}
}

impl Highlighter {
	pub fn to_line_widget(regions: Vec<(highlighting::Style, &str)>, indent: &str) -> Line<'static> {
		let spans: Vec<_> = regions
			.into_iter()
			.map(|(style, s)| {
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
					content: s.replace('\t', indent).into(),
					style:   ratatui::style::Style {
						fg: Self::to_ansi_color(style.foreground),
						// bg: Self::to_ansi_color(style.background),
						add_modifier: modifier,
						..Default::default()
					},
				}
			})
			.collect();

		Line::from(spans)
	}

	// Copy from https://github.com/sharkdp/bat/blob/master/src/terminal.rs
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
