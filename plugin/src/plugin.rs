use anyhow::Result;
use config::THEME;
use mlua::{Lua, LuaSerdeExt};
use shared::RoCell;

pub(crate) static LUA: RoCell<Lua> = RoCell::new();

pub fn init() {
	fn inner() -> Result<()> {
		let lua = Lua::new();
		lua.load(include_str!("../preset/utils.lua")).exec()?;
		lua.load(include_str!("../preset/inspect/inspect.lua")).exec()?;
		lua.load(include_str!("../preset/span.lua")).exec()?;
		lua.load(include_str!("../preset/line.lua")).exec()?;
		lua.load(include_str!("../preset/paragraph.lua")).exec()?;

		lua.load(include_str!("../preset/status.lua")).exec()?;

		lua.globals().set("THEME", lua.to_value(&*THEME)?)?;
		Ok(LUA.init(lua))
	}

	inner().expect("failed to initialize Lua");
}
