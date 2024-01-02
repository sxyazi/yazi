use mlua::{Lua, Table};

pub(super) struct Utils;

pub fn install(lua: &Lua) -> mlua::Result<()> {
	let ya: Table = lua.create_table()?;

	Utils::cache(lua, &ya)?;
	Utils::call(lua, &ya)?;
	Utils::image(lua, &ya)?;
	Utils::log(lua, &ya)?;
	Utils::plugin(lua, &ya)?;
	Utils::preview(lua, &ya)?;
	Utils::target(lua, &ya)?;
	Utils::time(lua, &ya)?;
	Utils::text(lua, &ya)?;
	#[cfg(unix)]
	Utils::unix_user(lua, &ya)?;

	lua.globals().set("ya", ya)
}
