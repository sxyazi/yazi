use mlua::Lua;

use crate::{bindings, elements};

pub fn slim_lua() -> mlua::Result<Lua> {
	let lua = Lua::new();

	// Base
	bindings::Cha::register(&lua)?;
	bindings::File::register(&lua, |_| {})?;
	crate::url::pour(&lua)?;

	crate::fs::install(&lua)?;
	crate::process::install(&lua)?;
	crate::utils::install(&lua)?;
	crate::Config::new(&lua).install_preview()?;

	// Elements
	let ui = lua.create_table()?;
	elements::Paragraph::install(&lua, &ui)?;
	elements::Rect::install(&lua, &ui)?;
	elements::Rect::register(&lua)?;
	lua.globals().set("ui", ui)?;

	Ok(lua)
}
