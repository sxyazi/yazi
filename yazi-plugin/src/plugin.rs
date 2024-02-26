use anyhow::Result;
use mlua::Lua;
use yazi_boot::BOOT;
use yazi_shared::RoCell;

pub static LUA: RoCell<Lua> = RoCell::new();

pub fn init() {
	fn stage_1(lua: &Lua) -> Result<()> {
		crate::Loader::init();
		crate::Config::new(lua).install_boot()?.install_manager()?.install_theme()?;
		crate::utils::init();
		crate::utils::install(lua)?;

		// Base
		lua.load(include_str!("../preset/inspect/inspect.lua")).exec()?;
		lua.load(include_str!("../preset/ya.lua")).exec()?;
		crate::bindings::Cha::register(lua)?;
		crate::bindings::File::register(lua)?;
		crate::bindings::Icon::register(lua)?;
		crate::elements::pour(lua)?;
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
		lua.load(include_str!("../preset/components/status.lua")).exec()?;

		Ok(())
	}

	fn stage_2(lua: &Lua) {
		lua.load(include_str!("../preset/setup.lua")).exec().unwrap();

		if let Ok(b) = std::fs::read(BOOT.config_dir.join("init.lua")) {
			lua.load(b).exec().unwrap();
		}
	}

	let lua = Lua::new();
	stage_1(&lua).expect("failed to initialize Lua");
	stage_2(&lua);
	LUA.init(lua);
}
