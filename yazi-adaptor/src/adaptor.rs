use std::{env, path::{Path, PathBuf}, sync::atomic::{AtomicBool, Ordering}};

use anyhow::{anyhow, Result};
use ratatui::prelude::Rect;
use tokio::{fs, sync::mpsc::UnboundedSender};
use tracing::warn;
use yazi_config::PREVIEW;
use yazi_shared::RoCell;

use super::{Iterm2, Kitty};
use crate::{ueberzug::Ueberzug, Sixel, TMUX};

static IMAGE_SHOWN: AtomicBool = AtomicBool::new(false);

#[allow(clippy::type_complexity)]
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

impl Adaptor {
	pub(super) fn detect() -> Self {
		let vars = [
			("ZELLIJ_SESSION_NAME", Self::Sixel),
			("KITTY_WINDOW_ID", Self::Kitty),
			("KONSOLE_VERSION", Self::Kitty),
			("ITERM_SESSION_ID", Self::Iterm2),
			("WEZTERM_EXECUTABLE", cfg!(windows).then_some(Self::Iterm2).unwrap_or(Self::Kitty)),
			("VSCODE_INJECTION", Self::Sixel),
		];
		match vars.iter().find(|v| env::var_os(v.0).is_some_and(|s| !s.is_empty())) {
			Some(var) => return var.1,
			None => warn!("[Adaptor] No special environment variables detected"),
		}

		let (term, program) = Self::term_program();
		match program.as_str() {
			"iTerm.app" => return Self::Iterm2,
			"WezTerm" => return cfg!(windows).then_some(Self::Iterm2).unwrap_or(Self::Kitty),
			"BlackBox" => return Self::Sixel,
			"vscode" => return Self::Sixel,
			"Hyper" => return Self::Sixel,
			"mintty" => return Self::Iterm2,
			_ => warn!("[Adaptor] Unknown TERM_PROGRAM: {program}"),
		}
		match term.as_str() {
			"xterm-kitty" => return Self::Kitty,
			"foot" => return Self::Sixel,
			"foot-extra" => return Self::Sixel,
			_ => warn!("[Adaptor] Unknown TERM: {term}"),
		}
		match env::var("XDG_SESSION_TYPE").unwrap_or_default().as_str() {
			"x11" => return Self::X11,
			"wayland" => return Self::Wayland,
			_ => warn!("[Adaptor] Could not identify XDG_SESSION_TYPE"),
		}
		if env::var_os("WAYLAND_DISPLAY").is_some_and(|s| !s.is_empty()) {
			return Self::Wayland;
		}
		if env::var_os("DISPLAY").is_some_and(|s| !s.is_empty()) {
			return Self::X11;
		if std::fs::symlink_metadata("/proc/sys/fs/binfmt_misc/WSLInterop").is_ok() {
			return Self::Kitty;
		}

		warn!("[Adaptor] WAYLAND_DISPLAY and DISPLAY are both empty");
		Self::Chafa
	}

	pub(super) fn term_program() -> (String, String) {
		fn tmux_env(name: &str) -> Result<String> {
			let output = std::process::Command::new("tmux").args(["show-environment", name]).output()?;

			String::from_utf8(output.stdout)?
				.trim()
				.strip_prefix(&format!("{}=", name))
				.map_or_else(|| Err(anyhow!("")), |s| Ok(s.to_string()))
		}

		let mut term = env::var("TERM").unwrap_or_default();
		let mut program = env::var("TERM_PROGRAM").unwrap_or_default();

		if *TMUX {
			term = tmux_env("TERM").unwrap_or(term);
			program = tmux_env("TERM_PROGRAM").unwrap_or(program);
		}

		(term, program)
	}
}

impl ToString for Adaptor {
	fn to_string(&self) -> String {
		match self {
			Self::Kitty => "kitty",
			Self::Iterm2 => "iterm2",
			Self::Sixel => "sixel",
			Self::X11 => "x11",
			Self::Wayland => "wayland",
			Self::Chafa => "chafa",
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
			Self::Kitty => Kitty::image_show(path, rect).await,
			Self::Iterm2 => Iterm2::image_show(path, rect).await,
			Self::Sixel => Sixel::image_show(path, rect).await,
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
			Self::Kitty => Kitty::image_hide(),
			Self::Iterm2 => Iterm2::image_hide(rect),
			Self::Sixel => Sixel::image_hide(rect),
			_ => Ok(if let Some(tx) = &*UEBERZUG {
				tx.send(None)?;
			}),
		}
	}

	#[inline]
	pub(super) fn needs_ueberzug(self) -> bool {
		!matches!(self, Self::Kitty | Self::Iterm2 | Self::Sixel)
	}
}
