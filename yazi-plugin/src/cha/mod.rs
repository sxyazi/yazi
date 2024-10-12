#![allow(clippy::module_inception)]

yazi_macro::mod_flat!(cha);

pub fn pour(lua: &mlua::Lua) -> mlua::Result<()> {
	cha::Cha::register(lua)?;
	cha::Cha::install(lua)?;

	Ok(())
}
