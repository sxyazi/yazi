use anyhow::Result;
use config::THEME;
use mlua::{Lua, LuaSerdeExt};
use shared::RoCell;

use crate::layout;

pub(crate) static LUA: RoCell<Lua> = RoCell::new();

pub fn init() {
	fn inner() -> Result<()> {
		let lua = Lua::new();

		// Base
		lua.load(include_str!("../preset/utils.lua")).exec()?;
		lua.load(include_str!("../preset/inspect/inspect.lua")).exec()?;

		// Elements
		lua.load(include_str!("../preset/elements/span.lua")).exec()?;
		lua.load(include_str!("../preset/elements/line.lua")).exec()?;
		lua.load(include_str!("../preset/elements/paragraph.lua")).exec()?;

		// Components
		lua.load(include_str!("../preset/components/status.lua")).exec()?;

		// Initialize
		lua.globals().set("THEME", lua.to_value(&*THEME)?)?;
		LUA.init(lua);

		layout::Layout::install()?;
		layout::Constraint::install()?;

		Ok(())
	}

	inner().expect("failed to initialize Lua");
}
