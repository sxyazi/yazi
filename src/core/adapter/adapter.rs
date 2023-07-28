use std::path::{Path, PathBuf};

use anyhow::Result;
use ratatui::prelude::Rect;
use tokio::sync::mpsc::UnboundedSender;

use super::{kitty::Kitty, ueberzug::Ueberzug};
use crate::config::{preview::PreviewAdapter, PREVIEW};

pub struct Adapter {
	ueberzug: Option<UnboundedSender<Option<(PathBuf, Rect)>>>,
}

impl Adapter {
	pub fn new() -> Self {
		let mut adapter = Self { ueberzug: None };

		if PREVIEW.adapter == PreviewAdapter::Ueberzug {
			adapter.ueberzug = Ueberzug::init().ok();
		}

		adapter
	}

	pub async fn image_show(&self, path: &Path, rect: Rect) -> Result<()> {
		match PREVIEW.adapter {
			PreviewAdapter::Kitty => Kitty::image_show(path, rect).await,
			PreviewAdapter::Ueberzug => {
				if let Some(tx) = &self.ueberzug {
					tx.send(Some((path.to_path_buf(), rect))).ok();
				}
				Ok(())
			}
		}
	}

	pub fn image_hide(&self) {
		match PREVIEW.adapter {
			PreviewAdapter::Kitty => Kitty::image_hide(),
			PreviewAdapter::Ueberzug => {
				if let Some(tx) = &self.ueberzug {
					tx.send(None).ok();
				}
			}
		}
	}
}
