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
use shared::RoCell;
use sixel::*;

pub use crate::image::*;

pub static ADAPTOR: RoCell<Adaptor> = RoCell::new();

// Tmux support
static TMUX: RoCell<bool> = RoCell::new();
static ESCAPE: RoCell<&'static str> = RoCell::new();
static START: RoCell<&'static str> = RoCell::new();
static CLOSE: RoCell<&'static str> = RoCell::new();

pub fn init() {
	TMUX.init(std::env::var_os("TMUX").is_some());
	START.init(if *TMUX { "\x1bPtmux;\x1b\x1b" } else { "\x1b" });
	CLOSE.init(if *TMUX { "\x1b\\" } else { "" });
	ESCAPE.init(if *TMUX { "\x1b\x1b" } else { "\x1b" });

	ADAPTOR.with(Default::default);
	ADAPTOR.start();
}
