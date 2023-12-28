use std::{mem, path::{Path, PathBuf}, sync::{atomic::{AtomicUsize, Ordering}, OnceLock}};

use anyhow::{anyhow, Result};
use crossterm::{style::{Color, Print, SetForegroundColor}, Command};
use syntect::{dumps::from_uncompressed_data, easy::HighlightLines, highlighting::{Style, Theme, ThemeSet}, parsing::{SyntaxReference, SyntaxSet}};
use tokio::{fs::File, io::{AsyncBufReadExt, BufReader}};
use yazi_config::THEME;
use yazi_shared::PeekError;

static INCR: AtomicUsize = AtomicUsize::new(0);
static SYNTECT_SYNTAX: OnceLock<SyntaxSet> = OnceLock::new();
static SYNTECT_THEME: OnceLock<Theme> = OnceLock::new();

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
			from_file().unwrap_or_else(|_| ThemeSet::load_defaults().themes["base16-ocean.dark"].clone())
		});
		let syntaxes =
			SYNTECT_SYNTAX.get_or_init(|| from_uncompressed_data(yazi_prebuild::syntaxes()).unwrap());

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

	pub async fn highlight(&self, skip: usize, limit: usize) -> Result<String, PeekError> {
		let mut reader = BufReader::new(File::open(&self.path).await?).lines();

		let syntax = Self::find_syntax(&self.path).await;
		let mut plain = syntax.is_err();

		let mut before = Vec::with_capacity(if plain { 0 } else { skip });
		let mut after = Vec::with_capacity(limit);

		let mut i = 0;
		while let Some(mut line) = reader.next_line().await? {
			i += 1;
			if i > skip + limit {
				break;
			}

			if !plain && line.len() > 6000 {
				mem::take(&mut before);
				plain = true;
			}

			if i > skip {
				line.push('\n');
				after.push(line);
			} else if !plain {
				line.push('\n');
				before.push(line);
			}
		}

		if skip > 0 && i < skip + limit {
			return Err(PeekError::Exceed(i.saturating_sub(limit)));
		}

		if plain {
			Ok(after.join(""))
		} else {
			Self::highlight_with(before, after, syntax.unwrap()).await
		}
	}

	async fn highlight_with(
		before: Vec<String>,
		after: Vec<String>,
		syntax: &'static SyntaxReference,
	) -> Result<String, PeekError> {
		let ticket = INCR.load(Ordering::Relaxed);

		tokio::task::spawn_blocking(move || {
			let (theme, syntaxes) = Self::init();
			let mut h = HighlightLines::new(syntax, theme);
			let mut result = String::new();

			for line in before {
				if ticket != INCR.load(Ordering::Relaxed) {
					return Err("Highlighting cancelled".into());
				}
				h.highlight_line(&line, syntaxes).map_err(|e| anyhow!(e))?;
			}
			for line in after {
				if ticket != INCR.load(Ordering::Relaxed) {
					return Err("Highlighting cancelled".into());
				}

				let regions = h.highlight_line(&line, syntaxes).map_err(|e| anyhow!(e))?;
				Self::write_highlighted_line_as_ansi(&regions, &mut result)
					.map_err(|e| anyhow!("Failed to write highlighted text to preview window: {}", e))?;
			}

			SetForegroundColor(Color::Reset)
				.write_ansi(&mut result)
				.map_err(|e| anyhow!("Failed to reset color at end of preview window line: {}", e))?;

			Ok(result)
		})
		.await?
	}

	fn write_highlighted_line_as_ansi(regions: &[(Style, &str)], s: &mut String) -> std::fmt::Result {
		for &(ref style, text) in regions {
			let fg = style.foreground;
			// As is done in the file previewer `bat`, we assume the convention that themes
			// with a 0x00 alpha value are actually 8-bit themes, with their colors in the
			// red color component. <https://github.com/sharkdp/bat/blob/b89dc15be1d86869bf73e5b9f96af7b930753a55/src/terminal.rs#L8>
			let foreground_color =
				if fg.a == 0 { Color::AnsiValue(fg.r) } else { Color::Rgb { r: fg.r, g: fg.g, b: fg.b } };

			SetForegroundColor(foreground_color).write_ansi(s)?;
			Print(text).write_ansi(s)?;
		}

		Ok(())
	}

	#[inline]
	pub fn abort() { INCR.fetch_add(1, Ordering::Relaxed); }
}
