use std::{env, path::{Path, PathBuf}, sync::atomic::{AtomicBool, Ordering}};

use anyhow::Result;
use config::PREVIEW;
use ratatui::prelude::Rect;
use shared::RoCell;
use tokio::{fs, sync::mpsc::UnboundedSender};

use super::{Iterm2, Kitty};
use crate::{ueberzug::Ueberzug, Sixel};

static IMAGE_SHOWN: AtomicBool = AtomicBool::new(false);
static UEBERZUG: RoCell<Option<UnboundedSender<Option<(PathBuf, Rect)>>>> = RoCell::new();

#[derive(Clone, Copy)]
pub enum Adaptor {
	Kitty,
	Iterm2,
	Sixel,

	// Supported by Überzug++
	X11,
	Wayland,
	Chafa,
}

impl Default for Adaptor {
	fn default() -> Self {
		let vars = [
			("KITTY_WINDOW_ID", Self::Kitty),
			("KONSOLE_VERSION", Self::Kitty),
			("ITERM_SESSION_ID", Self::Iterm2),
			("WEZTERM_EXECUTABLE", cfg!(windows).then_some(Self::Iterm2).unwrap_or(Self::Kitty)),
			("VSCODE_INJECTION", Self::Sixel),
		];
		if let Some(var) = vars.iter().find(|v| env::var_os(v.0).is_some()) {
			return var.1;
		}

		match env::var("TERM_PROGRAM").unwrap_or_default().as_str() {
			"iTerm.app" => return Self::Iterm2,
			"WezTerm" => return cfg!(windows).then_some(Self::Iterm2).unwrap_or(Self::Kitty),
			"BlackBox" => return Self::Sixel,
			"vscode" => return Self::Sixel,
			"Hyper" => return Self::Sixel,
			"mintty" => return Self::Iterm2,
			_ => {}
		}
		match env::var("TERM").unwrap_or_default().as_str() {
			"xterm-kitty" => return Self::Kitty,
			"foot" => return Self::Sixel,
			_ => {}
		}
		match env::var("XDG_SESSION_TYPE").unwrap_or_default().as_str() {
			"x11" => Self::X11,
			"wayland" => Self::Wayland,
			_ => Self::Chafa,
		}
	}
}

impl ToString for Adaptor {
	fn to_string(&self) -> String {
		match self {
			Adaptor::Kitty => "kitty",
			Adaptor::Iterm2 => "iterm2",
			Adaptor::Sixel => "sixel",
			Adaptor::X11 => "x11",
			Adaptor::Wayland => "wayland",
			Adaptor::Chafa => "chafa",
		}
		.to_string()
	}
}

impl Adaptor {
	pub(super) fn start(self) {
		UEBERZUG.init(if self.needs_ueberzug() { Ueberzug::start(self).ok() } else { None });
	}

	pub async fn image_show(self, mut path: &Path, rect: Rect) -> Result<()> {
		let cache = PREVIEW.cache(path, 0);
		if fs::metadata(&cache).await.is_ok() {
			path = cache.as_path();
		}

		self.image_hide(rect).ok();
		IMAGE_SHOWN.store(true, Ordering::Relaxed);

		match self {
			Adaptor::Kitty => Kitty::image_show(path, rect).await,
			Adaptor::Iterm2 => Iterm2::image_show(path, rect).await,
			Adaptor::Sixel => Sixel::image_show(path, rect).await,
			_ => Ok(if let Some(tx) = &*UEBERZUG {
				tx.send(Some((path.to_path_buf(), rect)))?;
			}),
		}
	}

	pub fn image_hide(self, rect: Rect) -> Result<()> {
		if !IMAGE_SHOWN.swap(false, Ordering::Relaxed) {
			return Ok(());
		}

		match self {
			Adaptor::Kitty => Kitty::image_hide(),
			Adaptor::Iterm2 => Iterm2::image_hide(rect),
			Adaptor::Sixel => Sixel::image_hide(rect),
			_ => Ok(if let Some(tx) = &*UEBERZUG {
				tx.send(None)?;
			}),
		}
	}

	#[inline]
	pub(super) fn needs_ueberzug(self) -> bool {
		!matches!(self, Adaptor::Kitty | Adaptor::Iterm2 | Adaptor::Sixel)
	}
}
