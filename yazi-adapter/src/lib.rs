yazi_macro::mod_pub!(drivers);

yazi_macro::mod_flat!(adapter brand dimension emulator icc image info mux unknown);

use yazi_shared::{RoCell, SyncCell, in_wsl};

pub static EMULATOR: RoCell<Emulator> = RoCell::new();
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
	let mut emulator = Emulator::detect().unwrap_or_default();
	TMUX.set(emulator.kind.is_left_and(|&b| b == Brand::Tmux));

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

	ADAPTOR.set(Adapter::matches(&EMULATOR));
	ADAPTOR.get().start();
	Ok(())
}
