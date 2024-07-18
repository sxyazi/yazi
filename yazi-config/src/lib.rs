#![allow(clippy::module_inception)]

use std::str::FromStr;

use yazi_shared::{RoCell, Xdg};

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
pub mod which;

pub use layout::*;
pub(crate) use pattern::*;
pub(crate) use preset::*;
pub use priority::*;

pub static LAYOUT: RoCell<arc_swap::ArcSwap<Layout>> = RoCell::new();

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

pub fn init() -> anyhow::Result<()> {
	let config_dir = Xdg::config_dir();
	let yazi_toml = &Preset::yazi(&config_dir)?;
	let keymap_toml = &Preset::keymap(&config_dir)?;
	let theme_toml = &Preset::theme(&config_dir)?;

	LAYOUT.with(<_>::default);

	KEYMAP.init(<_>::from_str(keymap_toml)?);
	LOG.init(<_>::from_str(yazi_toml)?);
	MANAGER.init(<_>::from_str(yazi_toml)?);
	OPEN.init(<_>::from_str(yazi_toml)?);
	PLUGIN.init(<_>::from_str(yazi_toml)?);
	PREVIEW.init(<_>::from_str(yazi_toml)?);
	TASKS.init(<_>::from_str(yazi_toml)?);
	THEME.init(<_>::from_str(theme_toml)?);
	INPUT.init(<_>::from_str(yazi_toml)?);
	SELECT.init(<_>::from_str(yazi_toml)?);
	WHICH.init(<_>::from_str(yazi_toml)?);

	// TODO: Remove in v0.3.2
	for c in &KEYMAP.manager {
		for r in &c.run {
			if r.name != "shell" {
				continue;
			}
			if !r.bool("confirm") && !r.bool("interactive") {
				let s = format!("`{}` ({})", c.on(), c.desc_or_run());
				eprintln!(
					r#"WARNING: In Yazi v0.3, the behavior of the interactive `shell` (i.e., shell templates) must be explicitly specified with either `--interactive` or `--confirm`.

Please replace e.g. `shell` with `shell --interactive`, `shell "my-template"` with `shell "my-template" --interactive`, in your keymap.toml for the key: {s}"#
				);
				return Ok(());
			} else if r.bool("confirm") && r.bool("interactive") {
				eprintln!(
					"The `shell` command cannot specify both `--confirm` and `--interactive` at the same time.",
				);
			}
		}
	}

	Ok(())
}
