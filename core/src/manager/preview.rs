use std::{io::BufRead, mem, path::{Path, PathBuf}, sync::{atomic::{AtomicUsize, Ordering}, Arc}};

use adaptor::Adaptor;
use anyhow::{anyhow, bail, Result};
use config::{BOOT, PREVIEW};
use crossterm::terminal::WindowSize;
use ratatui::prelude::Rect;
use shared::{MimeKind, Term};
use syntect::{easy::HighlightFile, util::as_24_bit_terminal_escaped};
use tokio::{fs, task::JoinHandle};

use super::{ALL_RATIO, CURRENT_RATIO, PARENT_RATIO, PREVIEW_BORDER, PREVIEW_MARGIN, PREVIEW_RATIO};
use crate::{emit, external, files::{Files, FilesOp}, highlighter};

#[derive(Default)]
pub struct Preview {
	pub lock: Option<(PathBuf, String)>,
	pub data: PreviewData,

	incr:   Arc<AtomicUsize>,
	handle: Option<JoinHandle<()>>,
}

#[derive(Debug, Default)]
pub enum PreviewData {
	#[default]
	None,
	Folder,
	Text(String),
	Image,
}

impl Preview {
	fn rect() -> Rect {
		let WindowSize { columns, rows, .. } = Term::size();

		let x = (columns as u32 * (PARENT_RATIO + CURRENT_RATIO) / ALL_RATIO) as u16;
		let width = (columns as u32 * PREVIEW_RATIO / ALL_RATIO) as u16;

		Rect {
			x:      x.saturating_add(PREVIEW_BORDER / 2),
			y:      PREVIEW_MARGIN / 2,
			width:  width.saturating_sub(PREVIEW_BORDER),
			height: rows.saturating_sub(PREVIEW_MARGIN),
		}
	}

	pub fn go(&mut self, path: &Path, mime: &str, show_image: bool) {
		let kind = MimeKind::new(mime);
		if !show_image && matches!(kind, MimeKind::Image | MimeKind::Video) {
			return;
		} else if self.same(path, mime) {
			return;
		} else {
			self.reset();
		}

		let (path, mime) = (path.to_path_buf(), mime.to_owned());
		let incr = self.incr.clone();

		self.handle = Some(tokio::spawn(async move {
			let result = match kind {
				MimeKind::Empty => Ok(PreviewData::None),
				MimeKind::Archive => Self::archive(&path).await.map(PreviewData::Text),
				MimeKind::Dir => Self::folder(&path).await,
				MimeKind::Image => Self::image(&path).await,
				MimeKind::Video => Self::video(&path).await,
				MimeKind::JSON => Self::json(&path).await.map(PreviewData::Text),
				MimeKind::PDF => Self::pdf(&path).await,
				MimeKind::Text => Self::highlight(&path, incr).await.map(PreviewData::Text),
				MimeKind::Others => Err(anyhow!("Unsupported mimetype: {mime}")),
			};

			emit!(Preview(path, mime, result.unwrap_or_default()));
		}));
	}

	pub fn reset(&mut self) -> bool {
		self.handle.take().map(|h| h.abort());
		self.incr.fetch_add(1, Ordering::Relaxed);
		Adaptor::image_hide(Self::rect()).ok();

		self.lock = None;
		!matches!(
			mem::replace(&mut self.data, PreviewData::None),
			PreviewData::None | PreviewData::Image
		)
	}

	pub fn reset_image(&mut self) -> bool {
		self.handle.take().map(|h| h.abort());
		self.incr.fetch_add(1, Ordering::Relaxed);
		Adaptor::image_hide(Self::rect()).ok();

		if matches!(self.data, PreviewData::Image) {
			self.lock = None;
			self.data = PreviewData::None;
		}
		false
	}

	pub async fn folder(path: &Path) -> Result<PreviewData> {
		emit!(Files(match Files::read_dir(path).await {
			Ok(items) => FilesOp::Read(path.to_path_buf(), items),
			Err(_) => FilesOp::IOErr(path.to_path_buf()),
		}));

		Ok(PreviewData::Folder)
	}

	pub async fn image(path: &Path) -> Result<PreviewData> {
		Adaptor::image_show(path, Self::rect()).await?;
		Ok(PreviewData::Image)
	}

	pub async fn video(path: &Path) -> Result<PreviewData> {
		let cache = BOOT.cache(path);
		if fs::metadata(&cache).await.is_err() {
			external::ffmpegthumbnailer(path, &cache).await?;
		}

		Self::image(&cache).await
	}

	pub async fn pdf(path: &Path) -> Result<PreviewData> {
		let cache = BOOT.cache(path);
		if fs::metadata(&cache).await.is_err() {
			external::pdftoppm(path, &cache).await?;
		}

		Self::image(&cache).await
	}

	pub async fn json(path: &Path) -> Result<String> {
		Ok(
			external::jq(path, Self::rect().height as usize)
				.await?
				.lines()
				.take(Self::rect().height as usize)
				.collect::<Vec<_>>()
				.join("\n"),
		)
	}

	pub async fn archive(path: &Path) -> Result<String> {
		Ok(
			external::lsar(path)
				.await?
				.into_iter()
				.take(Self::rect().height as usize)
				.map(|f| f.name)
				.collect::<Vec<_>>()
				.join("\n"),
		)
	}

	pub async fn highlight(path: &Path, incr: Arc<AtomicUsize>) -> Result<String> {
		let tick = incr.load(Ordering::Relaxed);
		let path = path.to_path_buf();
		let spaces = " ".repeat(PREVIEW.tab_size as usize);

		let (syntaxes, theme) = highlighter();
		tokio::task::spawn_blocking(move || -> Result<String> {
			let mut h = HighlightFile::new(path, syntaxes, theme)?;
			let mut line = String::new();
			let mut buf = String::new();

			let mut i = Self::rect().height as usize;
			while i > 0 && h.reader.read_line(&mut line)? > 0 {
				if tick != incr.load(Ordering::Relaxed) {
					bail!("Preview cancelled");
				}

				i -= 1;
				line = line.replace('\t', &spaces);
				let regions = h.highlight_lines.highlight_line(&line, syntaxes)?;
				buf.push_str(&as_24_bit_terminal_escaped(&regions, false));
				line.clear();
			}

			buf.push_str("\x1b[0m");
			Ok(buf)
		})
		.await?
	}
}

impl Preview {
	#[inline]
	pub fn same(&self, path: &Path, mime: &str) -> bool {
		self.lock.as_ref().map(|(p, m)| p == path && m == mime).unwrap_or(false)
	}

	#[inline]
	pub fn same_path(&self, path: &Path) -> bool {
		self.lock.as_ref().map(|(p, _)| p == path).unwrap_or(false)
	}
}
