use std::{path::{Path, PathBuf}, sync::atomic::{AtomicBool, Ordering}};

use anyhow::Result;
use config::{preview::PreviewAdaptor, BOOT, PREVIEW};
use once_cell::sync::Lazy;
use ratatui::prelude::Rect;
use tokio::{fs, sync::mpsc::UnboundedSender};

use super::{Iterm2, Kitty, Ueberzug};
use crate::Sixel;

static IMAGE_SHOWN: AtomicBool = AtomicBool::new(false);

#[allow(clippy::type_complexity)]
static UEBERZUG: Lazy<Option<UnboundedSender<Option<(PathBuf, Rect)>>>> =
	Lazy::new(|| if PREVIEW.adaptor.needs_ueberzug() { Ueberzug::start().ok() } else { None });

pub struct Adaptor;

impl Adaptor {
	pub fn init() { Lazy::force(&UEBERZUG); }

	pub async fn image_show(mut path: &Path, rect: Rect) -> Result<()> {
		if IMAGE_SHOWN.swap(true, Ordering::Relaxed) {
			Self::image_hide(rect).ok();
		}

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
		}
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
