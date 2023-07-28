use std::{fs::File, io::{BufRead, BufReader}, path::{Path, PathBuf}, sync::{Arc, OnceLock}};

use anyhow::{anyhow, Result};
use ratatui::prelude::Rect;
use syntect::{easy::HighlightFile, highlighting::{Theme, ThemeSet}, parsing::SyntaxSet, util::as_24_bit_terminal_escaped};
use tokio::{fs, task::JoinHandle};

use super::{ALL_RATIO, PREVIEW_BORDER, PREVIEW_PADDING, PREVIEW_RATIO};
use crate::{config::{PREVIEW, THEME}, core::{adapter::Adapter, external, files::{Files, FilesOp}, tasks::Precache}, emit, misc::{tty_size, MimeKind}};

static SYNTECT_SYNTAX: OnceLock<SyntaxSet> = OnceLock::new();
static SYNTECT_THEME: OnceLock<Theme> = OnceLock::new();

pub struct Preview {
	pub path: PathBuf,
	pub data: PreviewData,

	handle:  Option<JoinHandle<()>>,
	adaptor: Arc<Adapter>,
}

#[derive(Debug, Default)]
pub enum PreviewData {
	#[default]
	None,
	Folder,
	Text(String),
}

impl Preview {
	pub fn new() -> Self {
		Self {
			path:    Default::default(),
			data:    Default::default(),
			handle:  Default::default(),
			adaptor: Arc::new(Adapter::new()),
		}
	}

	fn size() -> (u16, u16) {
		let s = tty_size();
		let col = (s.ws_col as u32 * PREVIEW_RATIO / ALL_RATIO) as u16;
		(col.saturating_sub(PREVIEW_BORDER), s.ws_row.saturating_sub(PREVIEW_PADDING))
	}

	pub fn go(&mut self, path: &Path, mime: &str) {
		if let Some(handle) = self.handle.take() {
			handle.abort();
		}

		let adaptor = self.adaptor.clone();
		let (path, mime) = (path.to_path_buf(), mime.to_owned());
		self.handle = Some(tokio::spawn(async move {
			let result = match MimeKind::new(&mime) {
				MimeKind::Dir => Self::folder(&path).await,
				MimeKind::JSON => Self::json(&path).await.map(PreviewData::Text),
				MimeKind::Text => Self::highlight(&path).await.map(PreviewData::Text),
				MimeKind::Image => Self::image(adaptor, &path).await,
				MimeKind::Video => Self::video(adaptor, &path).await,
				MimeKind::Archive => Self::archive(&path).await.map(PreviewData::Text),
				MimeKind::Others => Err(anyhow!("Unsupported mimetype: {}", mime)),
			};

			emit!(Preview(path, result.unwrap_or_default()));
		}));
	}

	pub fn reset(&mut self) -> bool {
		if self.path == PathBuf::default() {
			return false;
		}

		self.path = Default::default();
		self.data = Default::default();
		true
	}

	pub async fn folder(path: &Path) -> Result<PreviewData> {
		emit!(Files(match Files::read_dir(&path).await {
			Ok(items) => FilesOp::Read(path.to_path_buf(), items),
			Err(_) => FilesOp::IOErr(path.to_path_buf()),
		}));

		Ok(PreviewData::Folder)
	}

	pub async fn image(adaptor: Arc<Adapter>, mut path: &Path) -> Result<PreviewData> {
		let cache = Precache::cache(path);
		if fs::metadata(&cache).await.is_ok() {
			path = cache.as_path();
		}

		// TODO: image
		adaptor.image_show(path, Rect {
			x:      todo!(),
			y:      todo!(),
			width:  todo!(),
			height: todo!(),
		});
		Ok(PreviewData::None)
	}

	pub async fn video(adaptor: Arc<Adapter>, path: &Path) -> Result<PreviewData> {
		let cache = Precache::cache(path);
		if fs::metadata(&cache).await.is_err() {
			external::ffmpegthumbnailer(path, &cache).await?;
		}

		Self::image(adaptor, &cache).await
	}

	pub async fn json(path: &Path) -> Result<String> {
		Ok(
			external::jq(path)
				.await?
				.lines()
				.take(Self::size().1 as usize)
				.collect::<Vec<_>>()
				.join("\n"),
		)
	}

	pub async fn archive(path: &Path) -> Result<String> {
		Ok(
			external::lsar(path)
				.await?
				.into_iter()
				.take(Self::size().1 as usize)
				.map(|f| f.name)
				.collect::<Vec<_>>()
				.join("\n"),
		)
	}

	pub async fn highlight(path: &Path) -> Result<String> {
		let syntax = SYNTECT_SYNTAX.get_or_init(|| SyntaxSet::load_defaults_newlines());
		let theme = SYNTECT_THEME.get_or_init(|| {
			let from_file = || -> Result<Theme> {
				let file = File::open(&THEME.preview.syntect_theme)?;
				Ok(ThemeSet::load_from_reader(&mut BufReader::new(file))?)
			};
			from_file().unwrap_or_else(|_| ThemeSet::load_defaults().themes["base16-ocean.dark"].clone())
		});

		let path = path.to_path_buf();
		let spaces = " ".repeat(PREVIEW.tab_size as usize);

		tokio::task::spawn_blocking(move || -> Result<String> {
			let mut h = HighlightFile::new(path, syntax, theme)?;
			let mut line = String::new();
			let mut buf = String::new();

			let mut i = Self::size().1 as usize;
			while i > 0 && h.reader.read_line(&mut line)? > 0 {
				i -= 1;
				line = line.replace('\t', &spaces);
				let regions = h.highlight_lines.highlight_line(&line, syntax)?;
				buf.push_str(&as_24_bit_terminal_escaped(&regions[..], false));
				line.clear();
			}

			buf.push_str("\x1b[0m");
			Ok(buf)
		})
		.await?
	}
}
