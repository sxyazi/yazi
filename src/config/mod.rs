use once_cell::sync::Lazy;

pub mod keymap;
pub mod manager;
pub mod open;
mod pattern;
mod preset;
pub mod preview;
pub mod theme;

pub(crate) use pattern::*;
pub(crate) use preset::*;

static MERGED_KEYMAP: Lazy<String> = Lazy::new(|| Preset::keymap());
static MERGED_THEME: Lazy<String> = Lazy::new(|| Preset::theme());
static MERGED_YAZI: Lazy<String> = Lazy::new(|| Preset::yazi());

pub static KEYMAP: Lazy<keymap::Keymap> = Lazy::new(|| keymap::Keymap::new());
pub static MANAGER: Lazy<manager::Manager> = Lazy::new(|| manager::Manager::new());
pub static OPEN: Lazy<open::Open> = Lazy::new(|| open::Open::new());
pub static PREVIEW: Lazy<preview::Preview> = Lazy::new(|| preview::Preview::new());
pub static THEME: Lazy<theme::Theme> = Lazy::new(|| theme::Theme::new());

pub fn init() {
	Lazy::force(&KEYMAP);
	Lazy::force(&MANAGER);
	Lazy::force(&OPEN);
	Lazy::force(&PREVIEW);
	Lazy::force(&THEME);
}
