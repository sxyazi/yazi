yazi_macro::mod_pub!(keymap mgr open opener plugin popup preview tasks theme which vfs);

yazi_macro::mod_flat!(icon layout mixing pattern platform preset priority selectable selector style utils yazi);

use std::io::{Read, Write};

use yazi_shim::{cell::{RoCell, SyncCell}, toml::{DeserializeOver, DeserializeOverWith}};
use yazi_tty::TTY;

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
		yazi = yazi.deserialize_over(&yazi::Yazi::read()?)?;
		keymap = keymap.deserialize_over(&keymap::Keymap::read()?)?;
	}

	YAZI.init(yazi);
	KEYMAP.init(keymap);
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
	THEME.init(build_flavor(light, merge)?);
	Ok(())
}

pub fn build_flavor(light: bool, merge: bool) -> anyhow::Result<theme::Theme> {
	let mut preset = Preset::theme(light)?;

	if merge {
		let theme_str = theme::Theme::read()?;
		let theme = toml::de::DeTable::parse(&theme_str)?;

		let flavor_str = theme::Flavor::from_theme(&theme, &theme_str)?.read(light)?;

		preset = preset.deserialize_over(&flavor_str)?;
		preset = error_with_input(
			preset.deserialize_over_with(toml::de::Deserializer::from(theme)),
			&theme_str,
		)?;
	}

	Ok(preset.reshape(light)?)
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

pub(crate) fn error_with_input<T>(
	result: Result<T, toml::de::Error>,
	input: &str,
) -> Result<T, toml::de::Error> {
	result.map_err(|mut err| {
		err.set_input(Some(input));
		err
	})
}
