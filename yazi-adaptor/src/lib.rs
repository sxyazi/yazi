#![allow(clippy::unit_arg)]

mod adaptor;
mod image;
mod iterm2;
mod kitty;
mod sixel;
mod ueberzug;

use adaptor::*;
use iterm2::*;
use kitty::*;
use sixel::*;
use yazi_shared::{env_exists, RoCell};

pub use crate::image::*;

pub static ADAPTOR: RoCell<Adaptor> = RoCell::new();

// Tmux support
static TMUX: RoCell<bool> = RoCell::new();
static ESCAPE: RoCell<&'static str> = RoCell::new();
static START: RoCell<&'static str> = RoCell::new();
static CLOSE: RoCell<&'static str> = RoCell::new();

pub fn init() {
	TMUX.init(env_exists("TMUX"));
	START.init(if *TMUX { "\x1bPtmux;\x1b\x1b" } else { "\x1b" });
	CLOSE.init(if *TMUX { "\x1b\\" } else { "" });
	ESCAPE.init(if *TMUX { "\x1b\x1b" } else { "\x1b" });

	ADAPTOR.init(Adaptor::detect());
	ADAPTOR.start();

	if *TMUX {
		_ = std::process::Command::new("tmux")
			.args(["set", "-p", "allow-passthrough", "on"])
			.stdin(std::process::Stdio::null())
			.stdout(std::process::Stdio::null())
			.stderr(std::process::Stdio::null())
			.spawn();
	}
}
