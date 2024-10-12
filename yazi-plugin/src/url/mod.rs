#![allow(clippy::module_inception)]

yazi_macro::mod_flat!(url);

pub fn pour(lua: &mlua::Lua) -> mlua::Result<()> {
	url::Url::register(lua)?;
	url::Url::install(lua)?;

	Ok(())
}
