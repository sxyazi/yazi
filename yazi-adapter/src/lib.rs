#![allow(clippy::unit_arg)]

yazi_macro::mod_pub!(drivers);

yazi_macro::mod_flat!(adapter brand dimension emulator image info mux unknown);

use yazi_shared::{RoCell, SyncCell, env_exists, in_wsl};

pub static EMULATOR: RoCell<Emulator> = RoCell::new();
pub static ADAPTOR: RoCell<Adapter> = RoCell::new();

// Image state
static SHOWN: SyncCell<Option<ratatui::layout::Rect>> = SyncCell::new(None);

// Tmux support
pub static TMUX: RoCell<u8> = RoCell::new();
static ESCAPE: RoCell<&'static str> = RoCell::new();
static START: RoCell<&'static str> = RoCell::new();
static CLOSE: RoCell<&'static str> = RoCell::new();

// WSL support
pub static WSL: RoCell<bool> = RoCell::new();

// Neovim support
pub static NVIM: RoCell<bool> = RoCell::new();

pub fn init() -> anyhow::Result<()> {
	// Tmux support
	TMUX.init(Mux::tmux_passthrough());
	ESCAPE.init(if *TMUX == 2 { "\x1b\x1b" } else { "\x1b" });
	START.init(if *TMUX == 2 { "\x1bPtmux;\x1b\x1b" } else { "\x1b" });
	CLOSE.init(if *TMUX == 2 { "\x1b\\" } else { "" });

	// WSL support
	WSL.init(in_wsl());

	// Neovim support
	NVIM.init(env_exists("NVIM_LOG_FILE") && env_exists("NVIM"));

	EMULATOR.init(Emulator::detect());
	yazi_config::init_flavor(EMULATOR.light)?;

	ADAPTOR.init(Adapter::matches(*EMULATOR));
	ADAPTOR.start();

	Ok(())
}
