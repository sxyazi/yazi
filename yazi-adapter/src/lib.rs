yazi_macro::mod_pub!(drivers);

yazi_macro::mod_flat!(adapter icc image);

use yazi_emulator::{Brand, CLOSE, EMULATOR, ESCAPE, Emulator, Mux, START, TMUX};
use yazi_shared::in_wsl;
use yazi_shim::cell::{RoCell, SyncCell};

pub static ADAPTOR: RoCell<Adapter> = RoCell::new();

// WSL support
pub static WSL: SyncCell<bool> = SyncCell::new(false);

pub fn init() -> anyhow::Result<()> {
	// WSL support
	WSL.set(in_wsl());

	// Emulator detection
	let mut emulator = Emulator::detect().unwrap_or_default();
	TMUX.set(emulator.kind.left() == Some(Brand::Tmux));

	// Tmux support
	if TMUX.get() {
		ESCAPE.set("\x1b\x1b");
		START.set("\x1bPtmux;\x1b\x1b");
		CLOSE.set("\x1b\\");
		Mux::tmux_passthrough();
		emulator = Emulator::detect().unwrap_or_default();
	}

	EMULATOR.init(emulator);
	yazi_config::init_flavor(EMULATOR.light)?;

	ADAPTOR.init(Adapter::new());
	ADAPTOR.start();
	Ok(())
}
