use anyhow::Result;
use mlua::{Lua, Table};
use shared::RoCell;

use crate::{bindings, layout, utils};

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
		lua.load(include_str!("../preset/components/folder.lua")).exec()?;
		lua.load(include_str!("../preset/components/header.lua")).exec()?;
		lua.load(include_str!("../preset/components/status.lua")).exec()?;

		// Initialize
		LUA.init(lua);
		GLOBALS.init(LUA.globals());
		utils::init()?;
		bindings::init()?;

		// Install
		crate::Config.install()?;

		layout::Constraint::install()?;
		layout::Gauge::install()?;
		layout::Layout::install()?;
		layout::Line::install()?;
		layout::List::install()?;
		layout::ListItem::install()?;
		layout::Paragraph::install()?;
		layout::Rect::install()?;
		layout::Span::install()?;
		layout::Style::install()?;

		Ok(())
	}

	inner().expect("failed to initialize Lua");
}
