#![allow(clippy::unit_arg, clippy::option_map_unit_fn)]

yazi_macro::mod_pub!(drivers);

yazi_macro::mod_flat!(adapter brand dimension emulator image info mux unknown);

use yazi_shared::{SyncCell, in_wsl};

pub static EMULATOR: SyncCell<Emulator> = SyncCell::new(Emulator::unknown());
pub static ADAPTOR: SyncCell<Adapter> = SyncCell::new(Adapter::Chafa);

// Image state
static SHOWN: SyncCell<Option<ratatui::layout::Rect>> = SyncCell::new(None);

// WSL support
pub static WSL: SyncCell<bool> = SyncCell::new(false);

// Tmux support
pub static TMUX: SyncCell<bool> = SyncCell::new(false);
static ESCAPE: SyncCell<&'static str> = SyncCell::new("\x1b");
static START: SyncCell<&'static str> = SyncCell::new("\x1b");
static CLOSE: SyncCell<&'static str> = SyncCell::new("");

pub fn init() -> anyhow::Result<()> {
	// WSL support
	WSL.set(in_wsl());

	// Emulator detection
	EMULATOR.set(Emulator::detect().unwrap_or_default());
	TMUX.set(EMULATOR.get().kind.is_left_and(|&b| b == Brand::Tmux));

	// Tmux support
	if TMUX.get() {
		ESCAPE.set("\x1b\x1b");
		START.set("\x1bPtmux;\x1b\x1b");
		CLOSE.set("\x1b\\");
		Mux::tmux_passthrough();
		EMULATOR.set(Emulator::detect().unwrap_or_default());
	}

	yazi_config::init_flavor(EMULATOR.get().light)?;

	ADAPTOR.set(Adapter::matches(EMULATOR.get()));
	ADAPTOR.get().start();
	Ok(())
}
