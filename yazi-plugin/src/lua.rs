use anyhow::{Context, Result};
use mlua::Lua;
use yazi_boot::BOOT;
use yazi_shared::RoCell;

use crate::runtime::Runtime;

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
	lua.load(include_str!("../preset/ya.lua")).set_name("ya.lua").exec()?;
	crate::bindings::Icon::register(lua)?;
	crate::bindings::MouseEvent::register(lua)?;
	crate::elements::pour(lua)?;
	crate::loader::install(lua)?;
	crate::pubsub::install(lua)?;
	crate::cha::pour(lua)?;
	crate::file::pour(lua)?;
	crate::url::pour(lua)?;

	// Components
	lua.load(include_str!("../preset/components/current.lua")).set_name("current.lua").exec()?;
	lua.load(include_str!("../preset/components/entity.lua")).set_name("entity.lua").exec()?;
	lua.load(include_str!("../preset/components/header.lua")).set_name("header.lua").exec()?;
	lua.load(include_str!("../preset/components/linemode.lua")).set_name("linemode.lua").exec()?;
	lua.load(include_str!("../preset/components/marker.lua")).set_name("marker.lua").exec()?;
	lua.load(include_str!("../preset/components/parent.lua")).set_name("parent.lua").exec()?;
	lua.load(include_str!("../preset/components/preview.lua")).set_name("preview.lua").exec()?;
	lua.load(include_str!("../preset/components/progress.lua")).set_name("progress.lua").exec()?;
	lua.load(include_str!("../preset/components/rail.lua")).set_name("rail.lua").exec()?;
	lua.load(include_str!("../preset/components/root.lua")).set_name("root.lua").exec()?;
	lua.load(include_str!("../preset/components/status.lua")).set_name("status.lua").exec()?;
	lua.load(include_str!("../preset/components/tab.lua")).set_name("tab.lua").exec()?;

	Ok(())
}

fn stage_2(lua: &'static Lua) -> mlua::Result<()> {
	lua.load(include_str!("../preset/setup.lua")).set_name("setup.lua").exec()?;
	lua.load(include_str!("../preset/compat.lua")).set_name("compat.lua").exec()?;

	if let Ok(b) = std::fs::read(BOOT.config_dir.join("init.lua")) {
		lua.load(b).set_name("init.lua").exec()?;
	}

	Ok(())
}
