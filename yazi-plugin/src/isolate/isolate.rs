use mlua::Lua;
use yazi_macro::plugin_preset as preset;

use crate::runtime::Runtime;

pub fn slim_lua(name: &str) -> mlua::Result<Lua> {
	let lua = Lua::new();
	lua.set_named_registry_value("rt", Runtime::new(name))?;
	crate::config::Config::new(&lua).install_preview()?.install_plugin()?;

	// Base
	let globals = lua.globals();
	globals.raw_set("ui", crate::elements::compose(&lua)?)?;
	globals.raw_set("ya", crate::utils::compose(&lua, true)?)?;
	globals.raw_set("fs", crate::fs::compose(&lua)?)?;

	crate::bindings::Cha::install(&lua)?;
	crate::file::pour(&lua)?;
	crate::url::pour(&lua)?;

	crate::loader::install_isolate(&lua)?;
	crate::process::install(&lua)?;

	// Addons
	lua.load(preset!("ya")).set_name("ya.lua").exec()?;

	Ok(lua)
}
