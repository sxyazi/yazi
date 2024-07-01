use std::{io::Cursor, mem, path::{Path, PathBuf}, sync::{atomic::{AtomicUsize, Ordering}, OnceLock}};

use anyhow::{anyhow, Result};
use ratatui::{prelude::Rect, text::{Line, Span, Text}};
use syntect::{dumps, easy::HighlightLines, highlighting::{self, Theme, ThemeSet}, parsing::{SyntaxReference, SyntaxSet}};
use tokio::{fs::File, io::{AsyncBufReadExt, BufReader}};
use unicode_width::UnicodeWidthStr;
use yazi_config::{PREVIEW, THEME};
use yazi_shared::PeekError;

static INCR: AtomicUsize = AtomicUsize::new(0);
static SYNTECT_SYNTAX: OnceLock<SyntaxSet> = OnceLock::new();
static SYNTECT_THEME: OnceLock<Theme> = OnceLock::new();

const MAX_LINE_BYTES_TO_PLAINTEXT_FALLBACK: usize = 6000;

pub struct Highlighter {
	path: PathBuf,
}

impl Highlighter {
	#[inline]
	pub fn new(path: &Path) -> Self { Self { path: path.to_owned() } }

	pub fn init() -> (&'static Theme, &'static SyntaxSet) {
		#[inline]
		fn from_file() -> Result<Theme> {
			let file = std::fs::File::open(&THEME.manager.syntect_theme)?;
			Ok(ThemeSet::load_from_reader(&mut std::io::BufReader::new(file))?)
		}

		let theme = SYNTECT_THEME.get_or_init(|| {
			from_file().unwrap_or_else(|_| {
				ThemeSet::load_from_reader(&mut Cursor::new(yazi_prebuild::ansi_theme())).unwrap()
			})
		});

		let syntaxes = SYNTECT_SYNTAX
			.get_or_init(|| dumps::from_uncompressed_data(yazi_prebuild::syntaxes()).unwrap());

		(theme, syntaxes)
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

	pub async fn highlight(
		&self,
		skip: usize,
		area: Rect,
		wrap: bool,
	) -> Result<Text<'static>, PeekError> {
		let mut reader = BufReader::new(File::open(&self.path).await?);

		let syntax = Self::find_syntax(&self.path).await;
		let mut plain = syntax.is_err();

		let mut before = Vec::with_capacity(if plain { 0 } else { skip });
		let mut after = Vec::with_capacity(area.height as usize);

		let mut lines_handled = 0;
		let mut long_line = vec![];
		while reader.read_until(b'\n', &mut long_line).await.is_ok() {
			if long_line.is_empty() || lines_handled > skip + area.height as usize {
				break;
			}
			if !plain && long_line.len() > MAX_LINE_BYTES_TO_PLAINTEXT_FALLBACK {
				plain = true;
				drop(mem::take(&mut before));
			}
			Self::replace_tabs_with_spaces(&mut long_line, PREVIEW.tab_size as usize);
			if wrap {
				Self::handle_line_wrap(
					&long_line,
					area,
					plain,
					skip,
					&mut lines_handled,
					&mut before,
					&mut after,
				);
			} else {
				lines_handled += 1;
				Self::handle_single_line(
					lines_handled,
					skip,
					plain,
					area.height as usize,
					String::from_utf8_lossy(&long_line).to_string(),
					&mut before,
					&mut after,
				);
			}
			long_line.clear();
		}

		let no_more_scroll = lines_handled < skip + area.height as usize;
		if skip > 0 && no_more_scroll {
			return Err(PeekError::Exceed(lines_handled.saturating_sub(area.height as usize)));
		}
		if plain {
			Ok(Text::from(after.join("")))
		} else {
			Self::highlight_with(before, after, syntax.unwrap()).await
		}
	}

	fn handle_line_wrap(
		long_line: &[u8],
		area: Rect,
		plain: bool,
		skip: usize,
		lines_handled: &mut usize,
		before: &mut Vec<String>,
		after: &mut Vec<String>,
	) {
		for line in Self::chunk_by_width(long_line, area.width as usize) {
			*lines_handled += 1;
			let must_break = Self::handle_single_line(
				*lines_handled,
				skip,
				plain,
				area.height as usize,
				line,
				before,
				after,
			);
			if must_break {
				break;
			}
		}
	}

	fn handle_single_line(
		lines_handled: usize,
		skip: usize,
		plain: bool,
		limit: usize,
		mut line: String,
		before: &mut Vec<String>,
		after: &mut Vec<String>,
	) -> bool {
		if line.is_empty() || lines_handled > skip + limit {
			return true;
		}

		if line.ends_with("\r\n") {
			line.pop();
			line.pop();
			line.push('\n');
		} else if !line.ends_with('\n') {
			line.push('\n');
		}

		if lines_handled > skip {
			after.push(line);
		} else if !plain {
			before.push(line);
		}
		false
	}

	fn chunk_by_width(line: &[u8], width: usize) -> Vec<String> {
		let line = String::from_utf8_lossy(line);
		if line.width() <= width {
			return vec![line.to_string()];
		}

		let mut resulted_lines = vec![];
		// Use this buffer to calculate width
		let mut buf_line = String::with_capacity(width);
		// Use this buffer to slice line
		let mut buf_chars = Vec::with_capacity(width);
		let mut last_break_char_idx = 0;
		let mut last_break_idx = 0;
		for (i, char) in line.chars().enumerate() {
			buf_line.push(char);
			buf_chars.push(char);

			if ",.; ".contains(char) {
				last_break_char_idx = i + 1
			}

			let buf_line_width = buf_line.width();
			if buf_line_width < width {
				continue;
			}

			if last_break_char_idx == 0 {
				// no spaces in line, break right here
				match buf_line_width.cmp(&width) {
					std::cmp::Ordering::Equal => {
						resulted_lines.push(mem::take(&mut buf_line));
						last_break_idx = i + 1;

						buf_line = String::with_capacity(width);
						buf_chars = Vec::with_capacity(width);
					}
					std::cmp::Ordering::Greater => {
						let last_idx = buf_line.len() - char.len_utf8();
						buf_line = buf_line[..last_idx].to_string();
						resulted_lines.push(mem::take(&mut buf_line));
						last_break_idx = i - last_idx + 1;

						buf_line = String::with_capacity(width);
						buf_line.push(char);
						buf_chars = Vec::with_capacity(width);
						buf_chars.push(char);
					}
					_ => {}
				}
			} else {
				let break_idx = last_break_char_idx - last_break_idx;
				resulted_lines.push(buf_chars[..break_idx].iter().collect());
				last_break_idx = last_break_char_idx;

				buf_chars = if last_break_char_idx == buf_chars.len() {
					Vec::with_capacity(width)
				} else {
					buf_chars[break_idx..].to_vec()
				};
				buf_line = buf_chars.iter().collect();
			}
			last_break_char_idx = 0;
		}
		if !buf_line.is_empty() && buf_line != "\n" {
			resulted_lines.push(buf_line);
		}

		resulted_lines
	}

	fn replace_tabs_with_spaces(buf: &mut Vec<u8>, tab_size: usize) {
		let mut i = 0;
		while i < buf.len() {
			if buf[i] == b'\t' {
				buf.splice(i..i + 1, vec![b' '; tab_size]);
				i += tab_size;
			} else {
				i += 1;
			}
		}
	}

	async fn highlight_with(
		before: Vec<String>,
		after: Vec<String>,
		syntax: &'static SyntaxReference,
	) -> Result<Text<'static>, PeekError> {
		let ticket = INCR.load(Ordering::Relaxed);

		tokio::task::spawn_blocking(move || {
			let (theme, syntaxes) = Self::init();
			let mut h = HighlightLines::new(syntax, theme);

			for line in before {
				if ticket != INCR.load(Ordering::Relaxed) {
					return Err("Highlighting cancelled".into());
				}
				h.highlight_line(&line, syntaxes).map_err(|e| anyhow!(e))?;
			}

			let mut lines = Vec::with_capacity(after.len());
			for line in after {
				if ticket != INCR.load(Ordering::Relaxed) {
					return Err("Highlighting cancelled".into());
				}

				let regions = h.highlight_line(&line, syntaxes).map_err(|e| anyhow!(e))?;
				lines.push(Self::to_line_widget(regions));
			}

			Ok(Text::from(lines))
		})
		.await?
	}

	#[inline]
	pub fn abort() { INCR.fetch_add(1, Ordering::Relaxed); }
}

impl Highlighter {
	// Copy from https://github.com/sharkdp/bat/blob/master/src/terminal.rs
	pub fn to_ansi_color(color: highlighting::Color) -> Option<ratatui::style::Color> {
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

	pub fn to_line_widget(regions: Vec<(highlighting::Style, &str)>) -> Line<'static> {
		let indent = " ".repeat(PREVIEW.tab_size as usize);
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
					content: s.replace('\t', &indent).into(),
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
}
