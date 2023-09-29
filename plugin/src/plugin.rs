use anyhow::Result;
use config::THEME;
use mlua::{Lua, LuaSerdeExt, SerializeOptions, Table};
use shared::RoCell;

use crate::{bindings, layout};

pub(crate) static LUA: RoCell<Lua> = RoCell::new();
pub(crate) static GLOBALS: RoCell<Table> = RoCell::new();

pub fn init() {
	fn inner() -> Result<()> {
		let lua = Lua::new();

		// Base
		lua.load(include_str!("../preset/ui.lua")).exec()?;
		lua.load(include_str!("../preset/utils.lua")).exec()?;
		lua.load(include_str!("../preset/inspect/inspect.lua")).exec()?;

		// Components
		lua.load(include_str!("../preset/components/status.lua")).exec()?;
		lua.load(include_str!("../preset/components/folder.lua")).exec()?;

		// Initialize
		LUA.init(lua);
		GLOBALS.init(LUA.globals());
		bindings::init()?;

		// Install
		layout::Constraint::install()?;
		layout::Layout::install()?;
		layout::Line::install()?;
		layout::Paragraph::install()?;
		layout::Span::install()?;
		layout::Style::install()?;

		let options =
			SerializeOptions::new().serialize_none_to_null(false).serialize_unit_to_null(false);
		GLOBALS.set("THEME", LUA.to_value_with(&*THEME, options)?)?;

		Ok(())
	}

	inner().expect("failed to initialize Lua");
}
