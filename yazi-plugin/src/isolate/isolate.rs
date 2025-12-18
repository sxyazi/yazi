use mlua::{IntoLua, Lua};
use yazi_binding::Runtime;
use yazi_macro::plugin_preset as preset;

pub fn slim_lua(name: &str) -> mlua::Result<Lua> {
	let lua = Lua::new();
	lua.set_app_data(Runtime::new_isolate(name));

	// Base
	let globals = lua.globals();
	globals.raw_set("ui", crate::elements::compose())?;
	globals.raw_set("ya", crate::utils::compose(true))?;
	globals.raw_set("fs", crate::fs::compose())?;
	globals.raw_set("rt", crate::runtime::compose())?;
	globals.raw_set("th", crate::theme::compose().into_lua(&lua)?)?;

	yazi_binding::Cha::install(&lua)?;
	yazi_binding::File::install(&lua)?;
	yazi_binding::Url::install(&lua)?;

	yazi_binding::Error::install(&lua)?;
	crate::loader::install(&lua)?;
	crate::process::install(&lua)?;

	// Addons
	lua.load(preset!("ya")).set_name("ya.lua").exec()?;

	Ok(lua)
}
