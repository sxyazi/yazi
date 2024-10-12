#![allow(clippy::module_inception)]

yazi_macro::mod_flat!(file);

pub fn pour(lua: &mlua::Lua) -> mlua::Result<()> {
	file::File::register(lua)?;
	file::File::install(lua)?;

	Ok(())
}
