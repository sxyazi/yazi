use anyhow::{Context, Result};
use futures::executor::block_on;
use mlua::Lua;
use yazi_binding::{Runtime, runtime_scope};
use yazi_fs::Xdg;
use yazi_macro::plugin_preset as preset;
use yazi_shim::cell::RoCell;

pub static LUA: RoCell<Lua> = RoCell::new();

pub(super) fn standard_lua() -> Result<Lua> {
	let lua = Lua::new();

	stage_1(&lua).context("Lua setup failed")?;
	stage_2(&lua).context("Lua runtime failed")?;

	Ok(lua)
}

fn stage_1(lua: &Lua) -> Result<()> {
	lua.set_app_data(Runtime::default());

	// Base
	let globals = lua.globals();
	globals.raw_set("ui", crate::elements::compose())?;
	globals.raw_set("ya", crate::utils::compose(false))?;
	globals.raw_set("fs", crate::fs::compose())?;
	globals.raw_set("ps", crate::pubsub::compose())?;
	globals.raw_set("rt", crate::runtime::compose())?;
	globals.raw_set("th", crate::theme::compose())?;

	yazi_binding::Error::install(lua)?;
	yazi_binding::Cha::install(lua)?;
	yazi_binding::process::install(lua)?;
	yazi_binding::File::install(lua)?;
	yazi_binding::Url::install(lua)?;
	yazi_binding::Path::install(lua)?;
	yazi_runner::loader::install(lua)?;

	// Addons
	lua.load(preset!("ya")).set_name("ya.lua").exec()?;

	// Components
	lua.load(preset!("components/current")).set_name("current.lua").exec()?;
	lua.load(preset!("components/entity")).set_name("entity.lua").exec()?;
	lua.load(preset!("components/header")).set_name("header.lua").exec()?;
	lua.load(preset!("components/linemode")).set_name("linemode.lua").exec()?;

	lua.load(preset!("components/marker")).set_name("marker.lua").exec()?;
	lua.load(preset!("components/markers")).set_name("markers.lua").exec()?;
	lua.load(preset!("components/modal")).set_name("modal.lua").exec()?;
	lua.load(preset!("components/parent")).set_name("parent.lua").exec()?;
	lua.load(preset!("components/preview")).set_name("preview.lua").exec()?;
	lua.load(preset!("components/progress")).set_name("progress.lua").exec()?;
	lua.load(preset!("components/rail")).set_name("rail.lua").exec()?;
	lua.load(preset!("components/rails")).set_name("rails.lua").exec()?;
	lua.load(preset!("components/root")).set_name("root.lua").exec()?;
	lua.load(preset!("components/status")).set_name("status.lua").exec()?;
	lua.load(preset!("components/tab")).set_name("tab.lua").exec()?;
	lua.load(preset!("components/tabs")).set_name("tabs.lua").exec()?;
	lua.load(preset!("components/tasks")).set_name("tasks.lua").exec()?;

	Ok(())
}

fn stage_2(lua: &Lua) -> mlua::Result<()> {
	lua.load(preset!("setup")).set_name("setup.lua").exec()?;
	lua.load(preset!("compat")).set_name("compat.lua").exec()?;

	if let Ok(b) = std::fs::read(Xdg::config_dir().join("init.lua")) {
		runtime_scope!(lua, "init", block_on(lua.load(b).set_name("init.lua").exec_async()))?;
	}

	Ok(())
}
