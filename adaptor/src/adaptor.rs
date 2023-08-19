use std::{
	path::{Path, PathBuf},
	sync::atomic::{AtomicBool, Ordering},
};

use anyhow::Result;
use config::{preview::PreviewAdaptor, BOOT, PREVIEW};
use ratatui::prelude::Rect;
use shared::RoCell;
use tokio::{fs, sync::mpsc::UnboundedSender};

use super::{Iterm2, Kitty};
use crate::Sixel;

static IMAGE_SHOWN: AtomicBool = AtomicBool::new(false);

#[allow(clippy::type_complexity)]
pub(super) static UEBERZUG: RoCell<Option<UnboundedSender<Option<(PathBuf, Rect)>>>> =
	RoCell::new();

pub struct Adaptor;

impl Adaptor {
	pub async fn image_show(mut path: &Path, rect: Rect) -> Result<()> {
		let cache = BOOT.cache(path);
		if fs::metadata(&cache).await.is_ok() {
			path = cache.as_path();
		}

		match PREVIEW.adaptor {
			PreviewAdaptor::Kitty => Kitty::image_show(path, rect).await,
			PreviewAdaptor::Iterm2 => Iterm2::image_show(path, rect).await,
			PreviewAdaptor::Sixel => Sixel::image_show(path, rect).await,
			_ => Ok(if let Some(tx) = &*UEBERZUG {
				tx.send(Some((path.to_path_buf(), rect)))?;
			}),
		}?;

		Ok(IMAGE_SHOWN.store(true, Ordering::Relaxed))
	}

	pub fn image_hide(rect: Rect) -> Result<()> {
		if !IMAGE_SHOWN.swap(false, Ordering::Relaxed) {
			return Ok(());
		}

		match PREVIEW.adaptor {
			PreviewAdaptor::Kitty => Kitty::image_hide(),
			PreviewAdaptor::Iterm2 => Iterm2::image_hide(rect),
			PreviewAdaptor::Sixel => Sixel::image_hide(rect),
			_ => Ok(if let Some(tx) = &*UEBERZUG {
				tx.send(None)?;
			}),
		}
	}
}
