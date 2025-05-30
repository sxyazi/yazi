#![allow(clippy::module_inception)]

yazi_macro::mod_pub!(keymap mgr open opener plugin popup preview tasks theme which);

yazi_macro::mod_flat!(layout pattern platform preset priority yazi);

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
		let dir = yazi_fs::Xdg::config_dir();
		yazi = yazi.deserialize_over(toml::Deserializer::new(&migrate(dir.join("yazi.toml"))))?;
		keymap = keymap.deserialize_over(toml::Deserializer::new(&migrate(dir.join("keymap.toml"))))?;
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
		let shadow = theme::Theme::deserialize_shadow(toml::Deserializer::new(&migrate(
			yazi_fs::Xdg::config_dir().join("theme.toml"),
		)))?;

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

// TODO: remove this in the future
fn migrate(p: std::path::PathBuf) -> String {
	let Ok(old) = std::fs::read_to_string(&p) else {
		return String::new();
	};
	let Ok(mut doc) = old.parse::<toml_edit::DocumentMut>() else {
		return old;
	};
	if doc.get("mgr").is_some() {
		return old;
	}
	let Some(manager) = doc.remove("manager") else {
		return old;
	};

	doc.insert("mgr", manager);
	let new = doc.to_string();

	let mut backup = p.clone();
	backup.set_file_name(format!(
		"{}-{}",
		p.file_name().unwrap().to_str().unwrap(),
		yazi_shared::timestamp_us()
	));

	if let Err(e) = std::fs::copy(&p, backup) {
		_ = TTY.writer().write_all(
			format!("WARNING: `[manager]` has been deprecated in favor of the new `[mgr]`, see #2803 for more details: https://github.com/sxyazi/yazi/pull/2803\r\n
Trying to migrate your config automatically failed, please edit the file manually, error while backuping {p:?}: {e}\r\n").as_bytes(),
		);
		return new;
	}

	if let Err(e) = std::fs::write(&p, &new) {
		_ = TTY.writer().write_all(
			format!("WARNING: `[manager]` has been deprecated in favor of the new `[mgr]`, see #2803 for more details: https://github.com/sxyazi/yazi/pull/2803\r\n
Trying to migrate your config automatically failed, please edit the file manually, error while writing {p:?}: {e}\r\n").as_bytes(),
		);
	}
	new
}
