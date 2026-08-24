yazi_macro::mod_pub!(keymap mgr open opener plugin popup preview tasks theme vfs which);

yazi_macro::mod_flat!(icon inject layout mixing pattern platform preset priority selectable selector tests yazi);

use std::io::Write;

use anyhow::Context;
use yazi_fs::Xdg;
use yazi_macro::writef;
use yazi_shim::{cell::{RoCell, SyncCell}, toml::{DeserializeOver, DeserializeOverWith}};
use yazi_term::TERM;
use yazi_tty::{TTY, sequence::SetSgr};

use crate::theme::{Flavor, Theme};

pub static YAZI: RoCell<yazi::Yazi> = RoCell::new();
pub static KEYMAP: RoCell<keymap::Keymap> = RoCell::new();
pub static THEME: RoCell<Theme> = RoCell::new();
pub static VFS: RoCell<vfs::Vfs> = RoCell::new();
pub static LAYOUT: SyncCell<Layout> = SyncCell::new(Layout::default());

pub fn setup() -> anyhow::Result<()> {
	if let Err(e) = try_init(true) {
		wait_for_key(e)?;
		try_init(false)?;
	}

	Ok(())
}

fn try_init(merge: bool) -> anyhow::Result<()> {
	let mut yazi = Preset::yazi()?;
	let mut keymap = Preset::keymap()?;
	let mut vfs = Preset::vfs()?;
	let mut theme = Preset::theme(false)?;

	if merge {
		yazi = parse("yazi.toml", yazi.deserialize_over(&yazi::Yazi::read()?))?;
		keymap = parse("keymap.toml", keymap.deserialize_over(&keymap::Keymap::read()?))?;
		vfs = parse("vfs.toml", vfs.deserialize_over(&vfs::Vfs::read()?))?;
		theme = parse("theme.toml", theme.deserialize_over(&Theme::read()?))?;
	} else {
		yazi = yazi.deserialize_over("")?;
		keymap = keymap.deserialize_over("")?;
		vfs = vfs.deserialize_over("")?;
		theme = theme.deserialize_over("")?;
	}

	YAZI.init(yazi);
	KEYMAP.init(keymap);
	VFS.init(vfs);
	THEME.init(theme.reshape(false)?);
	Ok(())
}

pub fn build_flavor(light: bool) -> anyhow::Result<Theme> {
	let mut preset = Preset::theme(light)?;
	let theme_str = Theme::read()?;
	let theme = parse("theme.toml", toml::de::DeTable::parse(&theme_str))?;

	let flavor_str = parse("theme.toml", Flavor::from_theme(&theme, &theme_str))?.read(light)?;

	preset = preset.deserialize_over(&flavor_str)?;
	preset = parse(
		"theme.toml",
		error_with_input(preset.deserialize_over_with(toml::de::Deserializer::from(theme)), &theme_str),
	)?;

	preset.reshape(light)
}

fn parse<T, E>(name: &str, result: Result<T, E>) -> anyhow::Result<T>
where
	E: std::error::Error + Send + Sync + 'static,
{
	result.with_context(|| format!("Failed to parse config {:?}", Xdg::config_dir().join(name)))
}

fn wait_for_key(e: anyhow::Error) -> anyhow::Result<()> {
	let mut stdout = TTY.lockout();

	write!(stdout, "{}\r\n", e.to_string().replace('\n', "\r\n"))?;
	if let Some(src) = e.source() {
		write!(stdout, "\r\nCaused by:\r\n{}\r\n", src.to_string().replace('\n', "\r\n"))?;
	}

	writef!(
		stdout,
		"{}{}Press any key to continue with preset settings...{}",
		SetSgr::Reverse,
		SetSgr::Bold,
		SetSgr::Reset
	)?;

	drop(stdout);
	TERM.source.try_poll(None, |event| event.is_key())?;
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
