use mlua::{IntoLua, Lua};
use yazi_macro::plugin_preset as preset;

pub fn slim_lua(lua: &Lua) -> mlua::Result<()> {
	// Base
	let globals = lua.globals();
	globals.raw_set("ui", crate::ui::compose())?;
	globals.raw_set("ya", crate::utils::compose(true))?;
	globals.raw_set("fs", crate::fs::compose())?;
	globals.raw_set("rt", crate::runtime::compose())?;
	globals.raw_set("km", crate::keymap::compose())?;
	globals.raw_set("th", crate::theme::compose().into_lua(lua)?)?;

	yazi_fs::cha::Cha::install(lua)?;
	yazi_fs::file::File::install(lua)?;
	yazi_shared::url::UrlBuf::install(lua)?;
	yazi_shared::path::PathBufDyn::install(lua)?;

	yazi_binding::Error::install(lua)?;
	yazi_binding::process::install(lua)?;
	yazi_runner::loader::install(lua)?;

	// Addons
	lua.load(preset!("ya")).set_name("ya.lua").exec()?;

	Ok(())
}
