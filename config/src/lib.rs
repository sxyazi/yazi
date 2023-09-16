#![allow(clippy::module_inception)]

use shared::RoCell;

mod bindings;
mod boot;
pub mod keymap;
mod log;
pub mod manager;
pub mod open;
mod pattern;
mod preset;
pub mod preview;
pub mod tasks;
pub mod theme;
mod validation;
mod xdg;

pub(crate) use pattern::*;
pub(crate) use preset::*;
pub(crate) use xdg::*;

static MERGED_KEYMAP: RoCell<String> = RoCell::new();
static MERGED_THEME: RoCell<String> = RoCell::new();
static MERGED_YAZI: RoCell<String> = RoCell::new();

pub static KEYMAP: RoCell<keymap::Keymap> = RoCell::new();
pub static LOG: RoCell<log::Log> = RoCell::new();
pub static MANAGER: RoCell<manager::Manager> = RoCell::new();
pub static OPEN: RoCell<open::Open> = RoCell::new();
pub static PREVIEW: RoCell<preview::Preview> = RoCell::new();
pub static TASKS: RoCell<tasks::Tasks> = RoCell::new();
pub static THEME: RoCell<theme::Theme> = RoCell::new();

pub static BOOT: RoCell<boot::Boot> = RoCell::new();

pub fn init() {
	MERGED_KEYMAP.with(Preset::keymap);
	MERGED_THEME.with(Preset::theme);
	MERGED_YAZI.with(Preset::yazi);

	KEYMAP.with(Default::default);
	LOG.with(Default::default);
	MANAGER.with(Default::default);
	OPEN.with(Default::default);
	PREVIEW.with(Default::default);
	TASKS.with(Default::default);
	THEME.with(Default::default);

	BOOT.with(Default::default);
}
