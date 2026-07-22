yazi_macro::mod_pub!(keymap mgr open opener plugin popup preview tasks theme vfs which);

yazi_macro::mod_flat!(icon inject layout mixing pattern platform preset priority selectable selector tests yazi);

use std::io::{Read, Write};

use anyhow::Context;
use yazi_macro::writef;
use yazi_shim::{cell::{RoCell, SyncCell}, toml::{DeserializeOver, DeserializeOverWith}};
use yazi_tty::{TTY, sequence::SetSgr};

pub static YAZI: RoCell<yazi::Yazi> = RoCell::new();
pub static KEYMAP: RoCell<keymap::Keymap> = RoCell::new();
pub static THEME: RoCell<theme::Theme> = RoCell::new();
pub static VFS: RoCell<vfs::Vfs> = RoCell::new();
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
	let mut vfs = Preset::vfs()?;

	if merge {
		let (p, s) = yazi::Yazi::read()?;
		yazi = yazi.deserialize_over(&s).with_context(|| format!("TOML parse error in {p:?}"))?;

		let (p, s) = keymap::Keymap::read()?;
		keymap = keymap.deserialize_over(&s).with_context(|| format!("TOML parse error in {p:?}"))?;

		let (p, s) = vfs::Vfs::read()?;
		vfs = vfs.deserialize_over(&s).with_context(|| format!("TOML parse error in {p:?}"))?;
	}

	YAZI.init(yazi);
	KEYMAP.init(keymap);
	VFS.init(vfs);
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
		let (theme_p, theme_str) = theme::Theme::read()?;
		let theme = toml::de::DeTable::parse(&theme_str)
			.with_context(|| format!("TOML parse error in {theme_p:?}"))?;

		let (flavor_p, flavor_str) = theme::Flavor::from_theme(&theme, &theme_str)
			.with_context(|| format!("TOML parse error in {theme_p:?}"))?
			.read(light)?;

		preset = preset.deserialize_over(&flavor_str).with_context(|| {
			format!("TOML parse error in {:?}", flavor_p.unwrap_or_else(|| theme_p.clone()))
		})?;
		preset = error_with_input(
			preset.deserialize_over_with(toml::de::Deserializer::from(theme)),
			&theme_str,
		)
		.with_context(|| format!("TOML parse error in {theme_p:?}"))?;
	}

	preset.reshape(light)
}

fn wait_for_key(e: anyhow::Error) -> anyhow::Result<()> {
	let mut stdout = &mut *TTY.lockout();

	writeln!(stdout, "{e}")?;
	if let Some(src) = e.source() {
		writeln!(stdout, "\nCaused by:\n{src}")?;
	}

	writef!(
		stdout,
		"{}{}Press <Enter> to continue with preset settings...{}",
		SetSgr::Reverse,
		SetSgr::Bold,
		SetSgr::Reset
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
