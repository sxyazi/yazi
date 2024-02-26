use mlua::Lua;

use crate::{bindings, elements};

pub fn slim_lua(name: &str) -> mlua::Result<Lua> {
	let lua = Lua::new();

	// Base
	bindings::Cha::register(&lua)?;
	bindings::File::register(&lua)?;
	crate::url::pour(&lua)?;

	crate::fs::install(&lua)?;
	crate::process::install(&lua)?;
	crate::utils::install(&lua)?;
	crate::Config::new(&lua).install_preview()?;
	lua.load(include_str!("../../preset/ya.lua")).exec()?;

	// Elements
	let ui = lua.create_table()?;
	elements::Line::install(&lua, &ui)?;
	elements::Paragraph::install(&lua, &ui)?;
	elements::Rect::register(&lua)?;
	elements::Rect::install(&lua, &ui)?;
	elements::Span::install(&lua, &ui)?;

	{
		let globals = lua.globals();
		globals.raw_set("ui", ui)?;
		globals.raw_set("YAZI_PLUGIN_NAME", lua.create_string(name)?)?;
		globals.raw_set("YAZI_SYNC_CALLS", 0)?;
	}

	Ok(lua)
}
