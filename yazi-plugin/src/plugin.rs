use anyhow::Result;
use mlua::{Lua, Table};
use yazi_shared::RoCell;

pub static LUA: RoCell<Lua> = RoCell::new();

pub fn init() {
	fn stage_1(lua: &Lua) -> Result<()> {
		crate::Loader::init();
		crate::Config::new(lua).install_theme()?.install_manager()?;
		crate::utils::install(lua)?;

		// Base
		lua.load(include_str!("../preset/inspect/inspect.lua")).exec()?;
		lua.load(include_str!("../preset/ui.lua")).exec()?;
		lua.load(include_str!("../preset/ya.lua")).exec()?;
		crate::elements::init(lua)?;

		// Components
		lua.load(include_str!("../preset/components/current.lua")).exec()?;
		lua.load(include_str!("../preset/components/folder.lua")).exec()?;
		lua.load(include_str!("../preset/components/header.lua")).exec()?;
		lua.load(include_str!("../preset/components/manager.lua")).exec()?;
		lua.load(include_str!("../preset/components/parent.lua")).exec()?;
		lua.load(include_str!("../preset/components/preview.lua")).exec()?;
		lua.load(include_str!("../preset/components/status.lua")).exec()?;

		Ok(())
	}

	fn stage_2(lua: &Lua) {
		let ya: Table = lua.globals().get("ya").unwrap();
		ya.set("SYNC_ON", true).unwrap();

		// TODO: plugin system
		// PLUGIN.preload.iter().for_each(|p| {
		// 	let b = std::fs::read(p).unwrap_or_else(|_| panic!("failed to read
		// plugin: {p:?}")); 	lua.load(&b).exec().unwrap_or_else(|_| panic!("failed
		// to load plugin: {p:?}")); });
	}

	let lua = Lua::new();
	stage_1(&lua).expect("failed to initialize Lua");
	stage_2(&lua);
	LUA.init(lua);
}
