use mlua::Lua;

use crate::{elements, runtime::Runtime};

pub fn slim_lua(name: &str) -> mlua::Result<Lua> {
	let lua = Lua::new();
	lua.set_named_registry_value("rt", Runtime::new(name))?;

	// Base
	crate::bindings::Icon::register(&lua)?;
	crate::cha::pour(&lua)?;
	crate::file::pour(&lua)?;
	crate::url::pour(&lua)?;

	crate::fs::install(&lua)?;
	crate::process::install(&lua)?;
	crate::utils::install_isolate(&lua)?;
	crate::Config::new(&lua).install_preview()?;
	lua.load(include_str!("../../preset/ya.lua")).set_name("ya.lua").exec()?;

	// Elements
	let ui = lua.create_table()?;
	elements::Line::install(&lua, &ui)?;
	elements::Paragraph::install(&lua, &ui)?;
	elements::Rect::register(&lua)?;
	elements::Rect::install(&lua, &ui)?;
	elements::Span::install(&lua, &ui)?;

	lua.globals().raw_set("ui", ui)?;
	Ok(lua)
}
