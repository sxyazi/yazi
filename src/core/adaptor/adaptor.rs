use std::{path::{Path, PathBuf}, sync::atomic::{AtomicBool, Ordering}};

use anyhow::Result;
use once_cell::sync::Lazy;
use ratatui::prelude::Rect;
use tokio::sync::mpsc::UnboundedSender;

use super::{iterm2::Iterm2, kitty::Kitty, ueberzug::Ueberzug};
use crate::config::{preview::PreviewAdaptor, PREVIEW};

static IMAGE_SHOWN: AtomicBool = AtomicBool::new(false);

static UEBERZUG: Lazy<Option<UnboundedSender<Option<(PathBuf, Rect)>>>> =
	Lazy::new(|| if PREVIEW.adaptor.needs_ueberzug() { Ueberzug::start().ok() } else { None });

pub struct Adaptor;

impl Adaptor {
	pub fn init() { Lazy::force(&UEBERZUG); }

	pub async fn image_show(path: &Path, rect: Rect) -> Result<()> {
		if IMAGE_SHOWN.swap(true, Ordering::Relaxed) {
			Self::image_hide(rect);
		}

		match PREVIEW.adaptor {
			PreviewAdaptor::Kitty => Kitty::image_show(path, rect).await,
			PreviewAdaptor::Iterm2 => Iterm2::image_show(path, rect).await,
			_ => {
				if let Some(tx) = &*UEBERZUG {
					tx.send(Some((path.to_path_buf(), rect))).ok();
				}
				Ok(())
			}
		}
	}

	pub fn image_hide(rect: Rect) {
		if !IMAGE_SHOWN.swap(false, Ordering::Relaxed) {
			return;
		}

		match PREVIEW.adaptor {
			PreviewAdaptor::Kitty => Kitty::image_hide(),
			PreviewAdaptor::Iterm2 => Iterm2::image_hide(rect),
			_ => {
				if let Some(tx) = &*UEBERZUG {
					tx.send(None).ok();
				}
			}
		}
	}
}
