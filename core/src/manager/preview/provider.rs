use std::{io::BufRead, path::Path, sync::atomic::{AtomicUsize, Ordering}};

use adaptor::Adaptor;
use anyhow::anyhow;
use config::{BOOT, MANAGER, PREVIEW};
use shared::{MimeKind, PagedError};
use syntect::{easy::HighlightFile, util::as_24_bit_terminal_escaped};
use tokio::fs;

use super::PreviewData;
use crate::{emit, external, files::{Files, FilesOp}, highlighter};

pub(super) struct Provider;

pub(super) static INCR: AtomicUsize = AtomicUsize::new(0);

impl Provider {
	pub(super) async fn auto(
		kind: MimeKind,
		path: &Path,
		skip: usize,
	) -> Result<PreviewData, PagedError> {
		match kind {
			MimeKind::Empty => Ok(PreviewData::None),
			MimeKind::Archive => Provider::archive(path, skip).await.map(PreviewData::Text),
			MimeKind::Dir => Provider::folder(path).await,
			MimeKind::Image => Provider::image(path).await,
			MimeKind::Video => Provider::video(path, skip).await,
			MimeKind::JSON => Provider::json(path, skip).await.map(PreviewData::Text),
			MimeKind::PDF => Provider::pdf(path, skip).await,
			MimeKind::Text => Provider::highlight(path, skip).await.map(PreviewData::Text),
			MimeKind::Others => Err("Unsupported mimetype".into()),
		}
	}

	pub(super) fn step_size(kind: MimeKind, step: usize) -> usize {
		match kind {
			MimeKind::Empty => 0,
			MimeKind::Archive => step * MANAGER.layout.preview_height() / 10,
			MimeKind::Dir => step * MANAGER.layout.preview_height() / 10,
			MimeKind::Image => 0,
			MimeKind::Video => step,
			MimeKind::JSON => step * MANAGER.layout.preview_height() / 10,
			MimeKind::PDF => 1,
			MimeKind::Text => step * MANAGER.layout.preview_height() / 10,
			MimeKind::Others => 0,
		}
	}

	pub(super) async fn folder(path: &Path) -> Result<PreviewData, PagedError> {
		emit!(Files(match Files::read_dir(path).await {
			Ok(items) => FilesOp::Read(path.to_path_buf(), items),
			Err(_) => FilesOp::IOErr(path.to_path_buf()),
		}));

		Ok(PreviewData::Folder)
	}

	pub(super) async fn image(path: &Path) -> Result<PreviewData, PagedError> {
		Adaptor::image_show(path, MANAGER.layout.preview_rect()).await?;
		Ok(PreviewData::Image)
	}

	pub(super) async fn video(path: &Path, skip: usize) -> Result<PreviewData, PagedError> {
		let cache = BOOT.cache(path, skip);
		if fs::metadata(&cache).await.is_err() {
			external::ffmpegthumbnailer(path, &cache, skip).await?;
		}

		Self::image(&cache).await
	}

	pub(super) async fn pdf(path: &Path, skip: usize) -> Result<PreviewData, PagedError> {
		let cache = BOOT.cache(path, skip);
		if fs::metadata(&cache).await.is_err() {
			external::pdftoppm(path, &cache, skip).await?;
		}

		Self::image(&cache).await
	}

	pub(super) async fn json(path: &Path, skip: usize) -> Result<String, PagedError> {
		external::jq(path, skip, MANAGER.layout.preview_height()).await
	}

	pub(super) async fn archive(path: &Path, skip: usize) -> Result<String, PagedError> {
		Ok(
			external::lsar(path, skip, MANAGER.layout.preview_height())
				.await?
				.into_iter()
				.map(|f| f.name)
				.collect::<Vec<_>>()
				.join("\n"),
		)
	}

	pub(super) async fn highlight(path: &Path, skip: usize) -> Result<String, PagedError> {
		let tick = INCR.load(Ordering::Relaxed);
		let path = path.to_path_buf();
		let spaces = " ".repeat(PREVIEW.tab_size as usize);

		let (syntaxes, theme) = highlighter();
		tokio::task::spawn_blocking(move || -> Result<String, PagedError> {
			let mut h = HighlightFile::new(path, syntaxes, theme)?;
			let mut line = String::new();
			let mut buf = String::new();

			let mut i = 0;
			let mut limit = MANAGER.layout.preview_height();
			while limit > 0 && h.reader.read_line(&mut line)? > 0 {
				if tick != INCR.load(Ordering::Relaxed) {
					return Err("Preview cancelled".into());
				}

				i += 1;
				if i <= skip {
					line.clear();
					continue;
				}

				limit -= 1;
				line = line.replace('\t', &spaces);
				let regions = h.highlight_lines.highlight_line(&line, syntaxes).map_err(|e| anyhow!(e))?;
				buf.push_str(&as_24_bit_terminal_escaped(&regions, false));
				line.clear();
			}

			if buf.is_empty() {
				return Err(PagedError::Exceed(i));
			}

			buf.push_str("\x1b[0m");
			Ok(buf)
		})
		.await?
	}
}
