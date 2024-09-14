use anyhow::{Context, Result};
use mlua::Lua;
use yazi_boot::BOOT;
use yazi_shared::RoCell;

use crate::{preset, runtime::Runtime};

pub static LUA: RoCell<Lua> = RoCell::new();

pub(super) fn init_lua() -> Result<()> {
	LUA.init(Lua::new());

	stage_1(&LUA).context("Lua setup failed")?;
	stage_2(&LUA).context("Lua runtime failed")?;
	Ok(())
}

fn stage_1(lua: &'static Lua) -> Result<()> {
	crate::Config::new(lua).install_boot()?.install_manager()?.install_theme()?;
	crate::utils::install(lua)?;

	// Base
	lua.set_named_registry_value("rt", Runtime::default())?;
	lua.load(preset!("ya")).set_name("ya.lua").exec()?;
	crate::bindings::Icon::register(lua)?;
	crate::bindings::MouseEvent::register(lua)?;
	crate::elements::pour(lua)?;
	crate::loader::install(lua)?;
	crate::pubsub::install(lua)?;
	crate::cha::pour(lua)?;
	crate::file::pour(lua)?;
	crate::url::pour(lua)?;

	// Components
	lua.load(preset!("components/current")).set_name("current.lua").exec()?;
	lua.load(preset!("components/entity")).set_name("entity.lua").exec()?;
	lua.load(preset!("components/header")).set_name("header.lua").exec()?;
	lua.load(preset!("components/linemode")).set_name("linemode.lua").exec()?;

	lua.load(preset!("components/marker")).set_name("marker.lua").exec()?;
	lua.load(preset!("components/parent")).set_name("parent.lua").exec()?;
	lua.load(preset!("components/preview")).set_name("preview.lua").exec()?;
	lua.load(preset!("components/progress")).set_name("progress.lua").exec()?;
	lua.load(preset!("components/rail")).set_name("rail.lua").exec()?;
	lua.load(preset!("components/root")).set_name("root.lua").exec()?;
	lua.load(preset!("components/status")).set_name("status.lua").exec()?;
	lua.load(preset!("components/tab")).set_name("tab.lua").exec()?;

	Ok(())
}

fn stage_2(lua: &'static Lua) -> mlua::Result<()> {
	lua.load(preset!("setup")).set_name("setup.lua").exec()?;
	lua.load(preset!("compat")).set_name("compat.lua").exec()?;

	if let Ok(b) = std::fs::read(BOOT.config_dir.join("init.lua")) {
		lua.load(b).set_name("init.lua").exec()?;
	}

	Ok(())
}
