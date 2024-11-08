#![allow(clippy::module_inception)]

use mlua::Lua;

yazi_macro::mod_flat!(cha);

pub fn pour(lua: &Lua) -> mlua::Result<()> {
	cha::Cha::register(lua)?;
	cha::Cha::install(lua)?;

	Ok(())
}
