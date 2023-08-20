use std::{io::BufRead, mem, path::{Path, PathBuf}, sync::{atomic::{AtomicUsize, Ordering}, Arc}};

use adaptor::Adaptor;
use anyhow::{anyhow, bail, Result};
use config::{BOOT, MANAGER, PREVIEW};
use shared::MimeKind;
use syntect::{easy::HighlightFile, util::as_24_bit_terminal_escaped};
use tokio::{fs, task::JoinHandle};

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
		Adaptor::image_hide(MANAGER.layout.preview_rect()).ok();

		self.lock = None;
		!matches!(
			mem::replace(&mut self.data, PreviewData::None),
			PreviewData::None | PreviewData::Image
		)
	}

	pub fn reset_image(&mut self) -> bool {
		self.handle.take().map(|h| h.abort());
		self.incr.fetch_add(1, Ordering::Relaxed);
		Adaptor::image_hide(MANAGER.layout.preview_rect()).ok();

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
		Adaptor::image_show(path, MANAGER.layout.preview_rect()).await?;
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
		external::jq(path, MANAGER.layout.preview_height()).await
	}

	pub async fn archive(path: &Path) -> Result<String> {
		Ok(
			external::lsar(path)
				.await?
				.into_iter()
				.take(MANAGER.layout.preview_height())
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

			let mut i = MANAGER.layout.preview_height();
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
