use anyhow::Result;
use mlua::Lua;
use yazi_boot::BOOT;
use yazi_shared::RoCell;

use crate::runtime::Runtime;

pub static LUA: RoCell<Lua> = RoCell::new();

pub(super) fn init_lua() {
	LUA.init(Lua::new());

	stage_1(&LUA).expect("Lua setup failed");
	stage_2(&LUA).expect("Lua runtime failed");
}

fn stage_1(lua: &'static Lua) -> Result<()> {
	crate::Config::new(lua).install_boot()?.install_manager()?.install_theme()?;
	crate::utils::install(lua)?;

	// Base
	lua.set_named_registry_value("rt", Runtime::default())?;
	lua.load(include_str!("../preset/ya.lua")).exec()?;
	crate::bindings::Icon::register(lua)?;
	crate::bindings::MouseEvent::register(lua)?;
	crate::elements::pour(lua)?;
	crate::loader::install(lua)?;
	crate::pubsub::install(lua)?;
	crate::cha::pour(lua)?;
	crate::file::pour(lua)?;
	crate::url::pour(lua)?;

	// Components
	lua.load(include_str!("../preset/components/current.lua")).exec()?;
	lua.load(include_str!("../preset/components/file.lua")).exec()?;
	lua.load(include_str!("../preset/components/folder.lua")).exec()?;
	lua.load(include_str!("../preset/components/header.lua")).exec()?;
	lua.load(include_str!("../preset/components/manager.lua")).exec()?;
	lua.load(include_str!("../preset/components/parent.lua")).exec()?;
	lua.load(include_str!("../preset/components/preview.lua")).exec()?;
	lua.load(include_str!("../preset/components/progress.lua")).exec()?;
	lua.load(include_str!("../preset/components/root.lua")).exec()?;
	lua.load(include_str!("../preset/components/status.lua")).exec()?;

	Ok(())
}

fn stage_2(lua: &'static Lua) -> mlua::Result<()> {
	lua.load(include_str!("../preset/setup.lua")).exec()?;

	if let Ok(b) = std::fs::read(BOOT.config_dir.join("init.lua")) {
		lua.load(b).exec()?;
	}

	Ok(())
}
