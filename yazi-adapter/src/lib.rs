#![allow(clippy::unit_arg)]

mod adapter;
mod chafa;
mod dimension;
mod emulator;
mod image;
mod iterm2;
mod kitty;
mod kitty_old;
mod sixel;
mod ueberzug;

pub use adapter::*;
use chafa::*;
pub use dimension::*;
pub use emulator::*;
use iterm2::*;
use kitty::*;
use kitty_old::*;
use sixel::*;
use ueberzug::*;
use yazi_shared::{env_exists, RoCell};

pub use crate::image::*;

pub static ADAPTOR: RoCell<Adapter> = RoCell::new();

// Tmux support
pub static TMUX: RoCell<bool> = RoCell::new();
static ESCAPE: RoCell<&'static str> = RoCell::new();
static START: RoCell<&'static str> = RoCell::new();
static CLOSE: RoCell<&'static str> = RoCell::new();

// Image state
static SHOWN: RoCell<arc_swap::ArcSwapOption<ratatui::layout::Rect>> = RoCell::new();

pub fn init() {
	TMUX.init(env_exists("TMUX") && env_exists("TMUX_PANE"));
	START.init(if *TMUX { "\x1bPtmux;\x1b\x1b" } else { "\x1b" });
	CLOSE.init(if *TMUX { "\x1b\\" } else { "" });
	ESCAPE.init(if *TMUX { "\x1b\x1b" } else { "\x1b" });

	if *TMUX {
		_ = std::process::Command::new("tmux")
			.args(["set", "-p", "allow-passthrough", "on"])
			.stdin(std::process::Stdio::null())
			.stdout(std::process::Stdio::null())
			.stderr(std::process::Stdio::null())
			.status();
	}

	SHOWN.with(<_>::default);

	ADAPTOR.init(Adapter::matches());
	ADAPTOR.start();
}
