use std::path::{Path, PathBuf};

use anyhow::Result;
use ratatui::prelude::Rect;
use tokio::sync::mpsc::UnboundedSender;

use super::{iterm2::Iterm2, kitty::Kitty, ueberzug::Ueberzug};
use crate::config::{preview::PreviewAdaptor, PREVIEW};

pub struct Adaptor {
	ueberzug: Option<UnboundedSender<Option<(PathBuf, Rect)>>>,
}

impl Adaptor {
	pub fn new() -> Self {
		let mut adaptor = Self { ueberzug: None };

		if PREVIEW.adaptor.needs_ueberzug() {
			adaptor.ueberzug = Ueberzug::init().ok();
		}

		adaptor
	}

	pub async fn image_show(&self, path: &Path, rect: Rect) -> Result<()> {
		match PREVIEW.adaptor {
			PreviewAdaptor::Kitty => Kitty::image_show(path, rect).await,
			PreviewAdaptor::Iterm2 => Iterm2::image_show(path, rect).await,
			_ => {
				if let Some(tx) = &self.ueberzug {
					tx.send(Some((path.to_path_buf(), rect))).ok();
				}
				Ok(())
			}
		}
	}

	pub fn image_hide(&self) {
		match PREVIEW.adaptor {
			PreviewAdaptor::Kitty => Kitty::image_hide(),
			PreviewAdaptor::Iterm2 => {}
			_ => {
				if let Some(tx) = &self.ueberzug {
					tx.send(None).ok();
				}
			}
		}
	}
}
