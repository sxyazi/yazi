use once_cell::sync::Lazy;

pub mod keymap;
pub mod manager;
pub mod open;
mod pattern;
pub mod preview;
pub mod theme;

pub(crate) use pattern::*;

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
