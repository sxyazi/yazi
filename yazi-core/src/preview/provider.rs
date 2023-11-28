use std::path::Path;

use tokio::fs;
use yazi_adaptor::ADAPTOR;
use yazi_config::{MANAGER, PREVIEW};
use yazi_scheduler::external;
use yazi_shared::{event::PreviewData, MimeKind, PeekError};

use crate::Highlighter;

pub(super) struct Provider;

impl Provider {
	pub(super) async fn auto(
		kind: MimeKind,
		path: &Path,
		skip: usize,
	) -> Result<PreviewData, PeekError> {
		match kind {
			MimeKind::Empty => Err("Empty file".into()),
			MimeKind::Archive => Provider::archive(path, skip).await.map(PreviewData::Text),
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
			MimeKind::Image => 0,
			MimeKind::Video => step,
			MimeKind::JSON => step * MANAGER.layout.preview_height() / 10,
			MimeKind::PDF => 1,
			MimeKind::Text => step * MANAGER.layout.preview_height() / 10,
			MimeKind::Others => step * MANAGER.layout.preview_height() / 10,
		}
	}

	pub(super) async fn image(path: &Path) -> Result<PreviewData, PeekError> {
		ADAPTOR.image_show(path, MANAGER.layout.image_rect()).await?;
		Ok(PreviewData::Image)
	}

	pub(super) async fn video(path: &Path, skip: usize) -> Result<PreviewData, PeekError> {
		let cache = PREVIEW.cache(path, skip);
		if fs::symlink_metadata(&cache).await.is_err() {
			external::ffmpegthumbnailer(path, &cache, skip).await?;
		}

		Self::image(&cache).await
	}

	pub(super) async fn pdf(path: &Path, skip: usize) -> Result<PreviewData, PeekError> {
		let cache = PREVIEW.cache(path, skip);
		if fs::symlink_metadata(&cache).await.is_err() {
			external::pdftoppm(path, &cache, skip).await?;
		}

		Self::image(&cache).await
	}

	pub(super) async fn json(path: &Path, skip: usize) -> Result<String, PeekError> {
		let result = external::jq(path, skip, MANAGER.layout.preview_height()).await;
		if let Err(PeekError::Unexpected(_)) = result {
			return Self::highlight(path, skip).await;
		}
		result
	}

	pub(super) async fn archive(path: &Path, skip: usize) -> Result<String, PeekError> {
		Ok(
			external::lsar(path, skip, MANAGER.layout.preview_height())
				.await?
				.into_iter()
				.map(|f| f.name)
				.collect::<Vec<_>>()
				.join("\n"),
		)
	}

	pub(super) async fn highlight(path: &Path, skip: usize) -> Result<String, PeekError> {
		let limit = MANAGER.layout.preview_height();
		let result = Highlighter::new(path.to_owned()).highlight(skip, limit).await?;
		Ok(result.replace('\t', &" ".repeat(PREVIEW.tab_size as usize)))
	}
}
