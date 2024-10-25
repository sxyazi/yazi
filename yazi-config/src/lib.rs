#![allow(clippy::module_inception)]

yazi_macro::mod_pub!(keymap log manager open plugin popup preview tasks theme which);

yazi_macro::mod_flat!(layout pattern preset priority);

use std::str::FromStr;

use yazi_shared::{RoCell, Xdg};

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
pub static CONFIRM: RoCell<popup::Confirm> = RoCell::new();
pub static PICK: RoCell<popup::Pick> = RoCell::new();
pub static WHICH: RoCell<which::Which> = RoCell::new();

fn try_init(merge: bool) -> anyhow::Result<()> {
	let (yazi_toml, keymap_toml, theme_toml) = if merge {
		let p = Xdg::config_dir();
		(Preset::yazi(&p)?, Preset::keymap(&p)?, Preset::theme(&p)?)
	} else {
		use yazi_macro::config_preset as preset;
		(preset!("yazi"), preset!("keymap"), preset!("theme"))
	};

	let keymap = <_>::from_str(&keymap_toml)?;
	let log = <_>::from_str(&yazi_toml)?;
	let manager = <_>::from_str(&yazi_toml)?;
	let open = <_>::from_str(&yazi_toml)?;
	let plugin = <_>::from_str(&yazi_toml)?;
	let preview = <_>::from_str(&yazi_toml)?;
	let tasks = <_>::from_str(&yazi_toml)?;
	let theme = <_>::from_str(&theme_toml)?;
	let input = <_>::from_str(&yazi_toml)?;
	let confirm = <_>::from_str(&yazi_toml)?;
	let pick = <_>::from_str(&yazi_toml)?;
	let which = <_>::from_str(&yazi_toml)?;

	LAYOUT.with(<_>::default);

	KEYMAP.init(keymap);
	LOG.init(log);
	MANAGER.init(manager);
	OPEN.init(open);
	PLUGIN.init(plugin);
	PREVIEW.init(preview);
	TASKS.init(tasks);
	THEME.init(theme);
	INPUT.init(input);
	CONFIRM.init(confirm);
	PICK.init(pick);
	WHICH.init(which);

	Ok(())
}

pub fn init() -> anyhow::Result<()> {
	if let Err(e) = try_init(true) {
		eprintln!("{e}");
		if let Some(src) = e.source() {
			eprintln!("\nCaused by:\n{src}");
		}

		use crossterm::style::{Attribute, Print, SetAttributes};
		crossterm::execute!(
			std::io::stderr(),
			SetAttributes(Attribute::Reverse.into()),
			SetAttributes(Attribute::Bold.into()),
			Print("Press <Enter> to continue with preset settings..."),
			SetAttributes(Attribute::Reset.into()),
			Print("\n"),
		)?;

		std::io::stdin().read_line(&mut String::new())?;
		try_init(false)?;
	}

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

	// TODO: Remove in v0.3.6
	if matches!(INPUT.create_title, popup::InputCreateTitle::One(_)) {
		eprintln!(
			r#"WARNING: The `create_title` under `[input]` now accepts an array instead of a string to support different titles for `create` and `create --dir` command.

Please change `create_title = "Create:"` to `create_title = ["Create:", "Create (dir):"]` in your yazi.toml.
"#
		);
	}

	Ok(())
}
