#![allow(clippy::module_inception)]

yazi_macro::mod_pub!(keymap mgr open plugin popup preview tasks theme which);

yazi_macro::mod_flat!(layout pattern preset priority);

use std::str::FromStr;

use yazi_shared::{RoCell, SyncCell};

pub static KEYMAP: RoCell<keymap::Keymap> = RoCell::new();
pub static MGR: RoCell<mgr::Mgr> = RoCell::new();
pub static OPEN: RoCell<open::Open> = RoCell::new();
pub static PLUGIN: RoCell<plugin::Plugin> = RoCell::new();
pub static PREVIEW: RoCell<preview::Preview> = RoCell::new();
pub static TASKS: RoCell<tasks::Tasks> = RoCell::new();
pub static THEME: RoCell<theme::Theme> = RoCell::new();
pub static INPUT: RoCell<popup::Input> = RoCell::new();
pub static CONFIRM: RoCell<popup::Confirm> = RoCell::new();
pub static PICK: RoCell<popup::Pick> = RoCell::new();
pub static WHICH: RoCell<which::Which> = RoCell::new();

pub static LAYOUT: SyncCell<Layout> = SyncCell::new(Layout::default());

pub fn init() -> anyhow::Result<()> {
	if let Err(e) = try_init(true) {
		wait_for_key(e)?;
		try_init(false)?;
	}

	// TODO: remove this
	for c in KEYMAP.mgr.iter().flat_map(|c| c.run.iter()) {
		if c.name == "arrow"
			&& c.first_str().unwrap_or_default().parse::<isize>().is_ok_and(|n| n <= -999 || n >= 999)
		{
			eprintln!("Deprecated command: `arrow -99999999` and `arrow 99999999` have been deprecated, please use `arrow top` and `arrow bot` instead, in your `keymap.toml`.

See #2294 for more details: https://github.com/sxyazi/yazi/pull/2294");
		}
	}

	Ok(())
}

pub fn init_flavor(light: bool) -> anyhow::Result<()> {
	let mut flavor_toml = Preset::flavor(light, true);
	if let Err(e) = flavor_toml {
		wait_for_key(e)?;
		flavor_toml = Preset::flavor(light, false);
	}

	let mut theme: theme::Theme = <_>::from_str(&flavor_toml.unwrap())?;
	theme.mgr.syntect_theme = theme
		.flavor
		.syntect_path(light)
		.unwrap_or_else(|| yazi_fs::expand_path(&theme.mgr.syntect_theme));

	THEME.init(theme);
	Ok(())
}

fn try_init(merge: bool) -> anyhow::Result<()> {
	let (yazi_toml, keymap_toml) = if merge {
		let p = yazi_fs::Xdg::config_dir();
		(Preset::yazi(&p)?, Preset::keymap(&p)?)
	} else {
		(yazi_macro::config_preset!("yazi"), yazi_macro::config_preset!("keymap"))
	};

	let keymap = <_>::from_str(&keymap_toml)?;
	let mgr = <_>::from_str(&yazi_toml)?;
	let open = <_>::from_str(&yazi_toml)?;
	let plugin = <_>::from_str(&yazi_toml)?;
	let preview = <_>::from_str(&yazi_toml)?;
	let tasks = <_>::from_str(&yazi_toml)?;
	let input = <_>::from_str(&yazi_toml)?;
	let confirm = <_>::from_str(&yazi_toml)?;
	let pick = <_>::from_str(&yazi_toml)?;
	let which = <_>::from_str(&yazi_toml)?;

	KEYMAP.init(keymap);
	MGR.init(mgr);
	OPEN.init(open);
	PLUGIN.init(plugin);
	PREVIEW.init(preview);
	TASKS.init(tasks);
	INPUT.init(input);
	CONFIRM.init(confirm);
	PICK.init(pick);
	WHICH.init(which);

	Ok(())
}

fn wait_for_key(e: anyhow::Error) -> anyhow::Result<()> {
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
	Ok(())
}
