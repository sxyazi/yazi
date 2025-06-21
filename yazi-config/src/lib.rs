#![allow(clippy::module_inception)]

yazi_macro::mod_pub!(keymap mgr open opener plugin popup preview tasks theme which);

yazi_macro::mod_flat!(color icon layout pattern platform preset priority style yazi);

use std::io::{Read, Write};

use yazi_shared::{RoCell, SyncCell};
use yazi_term::tty::TTY;

pub static YAZI: RoCell<yazi::Yazi> = RoCell::new();
pub static KEYMAP: RoCell<keymap::Keymap> = RoCell::new();
pub static THEME: RoCell<theme::Theme> = RoCell::new();
pub static LAYOUT: SyncCell<Layout> = SyncCell::new(Layout::default());

pub fn init() -> anyhow::Result<()> {
	if let Err(e) = try_init(true) {
		wait_for_key(e)?;
		try_init(false)?;
	}
	Ok(())
}

fn try_init(merge: bool) -> anyhow::Result<()> {
	let mut yazi = Preset::yazi()?;
	let mut keymap = Preset::keymap()?;

	if merge {
		yazi = yazi.deserialize_over(toml::Deserializer::new(&yazi::Yazi::read()?))?;
		keymap = keymap.deserialize_over(toml::Deserializer::new(&keymap::Keymap::read()?))?;
	}

	YAZI.init(yazi.reshape()?);
	KEYMAP.init(keymap.reshape()?);
	Ok(())
}

pub fn init_flavor(light: bool) -> anyhow::Result<()> {
	if let Err(e) = try_init_flavor(light, true) {
		wait_for_key(e)?;
		try_init_flavor(light, false)?;
	}
	Ok(())
}

fn try_init_flavor(light: bool, merge: bool) -> anyhow::Result<()> {
	let mut theme = Preset::theme(light)?;

	if merge {
		let shadow = theme::Theme::deserialize_shadow(toml::Deserializer::new(&theme::Theme::read()?))?;
		let flavor = shadow.flavor.as_ref().map(theme::Flavor::from).unwrap_or_default().read(light)?;
		theme = theme.deserialize_over(toml::Deserializer::new(&flavor))?;
		theme = theme.deserialize_over_with::<toml::Value>(shadow)?;
	}

	THEME.init(theme.reshape(light)?);
	Ok(())
}

fn wait_for_key(e: anyhow::Error) -> anyhow::Result<()> {
	let stdout = &mut *TTY.lockout();

	writeln!(stdout, "{e}")?;
	if let Some(src) = e.source() {
		writeln!(stdout, "\nCaused by:\n{src}")?;
	}

	use crossterm::style::{Attribute, Print, SetAttributes};
	crossterm::execute!(
		stdout,
		SetAttributes(Attribute::Reverse.into()),
		SetAttributes(Attribute::Bold.into()),
		Print("Press <Enter> to continue with preset settings..."),
		SetAttributes(Attribute::Reset.into()),
		Print("\n"),
	)?;

	TTY.reader().read_exact(&mut [0])?;
	Ok(())
}
