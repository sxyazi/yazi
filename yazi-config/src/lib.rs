#![allow(clippy::module_inception)]

use yazi_shared::{RoCell, Xdg};

pub mod headsup;
pub mod keymap;
mod layout;
mod log;
pub mod manager;
pub mod open;
mod pattern;
pub mod plugin;
pub mod popup;
mod preset;
pub mod preview;
mod priority;
mod tasks;
pub mod theme;
mod validation;
pub mod which;

pub use layout::*;
pub(crate) use pattern::*;
pub(crate) use preset::*;
pub use priority::*;

// TODO: remove this once Yazi 0.3 is released --
pub static DEPRECATED_EXEC: std::sync::atomic::AtomicBool =
	std::sync::atomic::AtomicBool::new(false);

static MERGED_YAZI: RoCell<String> = RoCell::new();
static MERGED_KEYMAP: RoCell<String> = RoCell::new();
static MERGED_THEME: RoCell<String> = RoCell::new();

pub static LAYOUT: RoCell<arc_swap::ArcSwap<Layout>> = RoCell::new();

pub static HEADSUP: RoCell<headsup::Headsup> = RoCell::new();
pub static KEYMAP: RoCell<keymap::Keymap> = RoCell::new();
pub static LOG: RoCell<log::Log> = RoCell::new();
pub static MANAGER: RoCell<manager::Manager> = RoCell::new();
pub static OPEN: RoCell<open::Open> = RoCell::new();
pub static PLUGIN: RoCell<plugin::Plugin> = RoCell::new();
pub static PREVIEW: RoCell<preview::Preview> = RoCell::new();
pub static TASKS: RoCell<tasks::Tasks> = RoCell::new();
pub static THEME: RoCell<theme::Theme> = RoCell::new();
pub static INPUT: RoCell<popup::Input> = RoCell::new();
pub static SELECT: RoCell<popup::Select> = RoCell::new();
pub static WHICH: RoCell<which::Which> = RoCell::new();

pub fn init() {
	let config_dir = Xdg::config_dir();
	MERGED_YAZI.init(Preset::yazi(&config_dir));
	MERGED_KEYMAP.init(Preset::keymap(&config_dir));
	MERGED_THEME.init(Preset::theme(&config_dir));

	LAYOUT.with(Default::default);

	HEADSUP.with(Default::default);
	KEYMAP.with(Default::default);
	LOG.with(Default::default);
	MANAGER.with(Default::default);
	OPEN.with(Default::default);
	PLUGIN.with(Default::default);
	PREVIEW.with(Default::default);
	TASKS.with(Default::default);
	THEME.with(Default::default);
	INPUT.with(Default::default);
	SELECT.with(Default::default);
	WHICH.with(Default::default);

	// TODO: remove this once Yazi 0.3 is released --
	if !HEADSUP.disable_exec_warn && DEPRECATED_EXEC.load(std::sync::atomic::Ordering::Relaxed) {
		println!(
			r#"
WARNING: `exec` will be deprecated in the next major version v0.3 and replaced by `run`.

Please replace all `exec = ...` with `run = ...`, in your `yazi.toml` and `keymap.toml`.

---
Add `disable_exec_warn = true` to your `yazi.toml` under `[headsup]` to suppress this warning.
"#
		);
	}
}
