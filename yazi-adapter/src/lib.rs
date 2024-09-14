#![allow(clippy::unit_arg)]

mod adapter;
mod chafa;
mod dimension;
mod emulator;
mod iip;
mod image;
mod kitty;
mod kitty_old;
mod sixel;
mod ueberzug;

pub use adapter::*;
use chafa::*;
pub use dimension::*;
pub use emulator::*;
use iip::*;
use kitty::*;
use kitty_old::*;
use sixel::*;
use ueberzug::*;
use yazi_shared::{env_exists, in_wsl, RoCell};

pub use crate::image::*;

pub static ADAPTOR: RoCell<Adapter> = RoCell::new();

// Tmux support
pub static TMUX: RoCell<bool> = RoCell::new();
static ESCAPE: RoCell<&'static str> = RoCell::new();
static START: RoCell<&'static str> = RoCell::new();
static CLOSE: RoCell<&'static str> = RoCell::new();

// WSL support
pub static WSL: RoCell<bool> = RoCell::new();

// Image state
static SHOWN: RoCell<arc_swap::ArcSwapOption<ratatui::layout::Rect>> = RoCell::new();

pub fn init() {
	// Tmux support
	TMUX.init(env_exists("TMUX") && env_exists("TMUX_PANE"));
	ESCAPE.init(if *TMUX { "\x1b\x1b" } else { "\x1b" });
	START.init(if *TMUX { "\x1bPtmux;\x1b\x1b" } else { "\x1b" });
	CLOSE.init(if *TMUX { "\x1b\\" } else { "" });

	if *TMUX {
		_ = std::process::Command::new("tmux")
			.args(["set", "-p", "allow-passthrough", "all"])
			.stdin(std::process::Stdio::null())
			.stdout(std::process::Stdio::null())
			.stderr(std::process::Stdio::null())
			.status();
	}

	// WSL support
	WSL.init(in_wsl());

	// Image state
	SHOWN.with(<_>::default);

	ADAPTOR.init(Adapter::matches());
	ADAPTOR.start();
}

pub fn tcsi(s: &str) -> std::borrow::Cow<str> {
	if *TMUX {
		std::borrow::Cow::Owned(format!(
			"{}{}{}",
			*START,
			s.trim_start_matches('\x1b').replace('\x1b', *ESCAPE),
			*CLOSE
		))
	} else {
		std::borrow::Cow::Borrowed(s)
	}
}
