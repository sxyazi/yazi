#![allow(clippy::module_inception)]

use mlua::Lua;

yazi_macro::mod_flat!(url);

pub fn pour(lua: &Lua) -> mlua::Result<()> {
	url::Url::register(lua)?;
	url::Url::install(lua)?;

	Ok(())
}
