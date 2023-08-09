#![allow(clippy::module_inception)]

use once_cell::sync::Lazy;

pub mod keymap;
mod log;
pub mod manager;
pub mod open;
mod pattern;
mod preset;
pub mod preview;
pub mod theme;

pub(crate) use pattern::*;
pub(crate) use preset::*;

static MERGED_KEYMAP: Lazy<String> = Lazy::new(Preset::keymap);
static MERGED_THEME: Lazy<String> = Lazy::new(Preset::theme);
static MERGED_YAZI: Lazy<String> = Lazy::new(Preset::yazi);

pub static KEYMAP: Lazy<keymap::Keymap> = Lazy::new(Default::default);
pub static LOG: Lazy<log::Log> = Lazy::new(Default::default);
pub static MANAGER: Lazy<manager::Manager> = Lazy::new(Default::default);
pub static OPEN: Lazy<open::Open> = Lazy::new(Default::default);
pub static PREVIEW: Lazy<preview::Preview> = Lazy::new(Default::default);
pub static THEME: Lazy<theme::Theme> = Lazy::new(Default::default);

pub fn init() {
	Lazy::force(&KEYMAP);
	Lazy::force(&LOG);
	Lazy::force(&MANAGER);
	Lazy::force(&OPEN);
	Lazy::force(&PREVIEW);
	Lazy::force(&THEME);
}
