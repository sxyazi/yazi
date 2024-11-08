#![allow(clippy::module_inception)]

use mlua::Lua;

yazi_macro::mod_flat!(file);

pub fn pour(lua: &Lua) -> mlua::Result<()> {
	file::File::register(lua)?;
	file::File::install(lua)?;

	Ok(())
}
