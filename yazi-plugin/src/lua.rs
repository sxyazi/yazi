use anyhow::{Context, Result};
use futures::executor::block_on;
use mlua::Lua;
use yazi_binding::{Runtime, runtime_mut};
use yazi_boot::BOOT;
use yazi_macro::plugin_preset as preset;
use yazi_shared::RoCell;

pub static LUA: RoCell<Lua> = RoCell::new();

pub(super) fn init_lua() -> Result<()> {
	LUA.init(Lua::new());

	stage_1(&LUA).context("Lua setup failed")?;
	stage_2(&LUA).context("Lua runtime failed")?;
	Ok(())
}

fn stage_1(lua: &'static Lua) -> Result<()> {
	lua.set_app_data(Runtime::new());

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
	crate::loader::install(lua)?;
	crate::process::install(lua)?;
	yazi_binding::File::install(lua)?;
	yazi_binding::Url::install(lua)?;

	// Addons
	lua.load(preset!("ya")).set_name("ya.lua").exec()?;

	// Components
	lua.load(preset!("components/current")).set_name("current.lua").exec()?;
	lua.load(preset!("components/entity")).set_name("entity.lua").exec()?;
	lua.load(preset!("components/header")).set_name("header.lua").exec()?;
	lua.load(preset!("components/linemode")).set_name("linemode.lua").exec()?;

	lua.load(preset!("components/marker")).set_name("marker.lua").exec()?;
	lua.load(preset!("components/modal")).set_name("modal.lua").exec()?;
	lua.load(preset!("components/parent")).set_name("parent.lua").exec()?;
	lua.load(preset!("components/preview")).set_name("preview.lua").exec()?;
	lua.load(preset!("components/progress")).set_name("progress.lua").exec()?;
	lua.load(preset!("components/rail")).set_name("rail.lua").exec()?;
	lua.load(preset!("components/root")).set_name("root.lua").exec()?;
	lua.load(preset!("components/status")).set_name("status.lua").exec()?;
	lua.load(preset!("components/tab")).set_name("tab.lua").exec()?;
	lua.load(preset!("components/tabs")).set_name("tabs.lua").exec()?;
	lua.load(preset!("components/tasks")).set_name("tasks.lua").exec()?;

	Ok(())
}

fn stage_2(lua: &'static Lua) -> mlua::Result<()> {
	lua.load(preset!("setup")).set_name("setup.lua").exec()?;
	lua.load(preset!("compat")).set_name("compat.lua").exec()?;

	if let Ok(b) = std::fs::read(BOOT.config_dir.join("init.lua")) {
		block_on(lua.load(b).set_name("init.lua").exec_async())?;
	}

	runtime_mut!(lua)?.initing = false;
	Ok(())
}
