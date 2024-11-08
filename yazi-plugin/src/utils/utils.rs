use mlua::Lua;

#[cfg(unix)]
pub(super) static HOSTNAME_CACHE: std::sync::OnceLock<Option<String>> = std::sync::OnceLock::new();

pub(super) struct Utils;

pub fn install(lua: &'static Lua) -> mlua::Result<()> {
	let ya = lua.create_table()?;

	Utils::app(lua, &ya)?;
	Utils::cache(lua, &ya)?;
	Utils::call(lua, &ya)?;
	Utils::image(lua, &ya)?;
	Utils::layer(lua, &ya)?;
	Utils::log(lua, &ya)?;
	Utils::preview(lua, &ya)?;
	Utils::sync(lua, &ya)?;
	Utils::target(lua, &ya)?;
	Utils::text(lua, &ya)?;
	Utils::time(lua, &ya)?;
	Utils::user(lua, &ya)?;

	lua.globals().raw_set("ya", ya)
}

pub fn install_isolate(lua: &Lua) -> mlua::Result<()> {
	let ya = lua.create_table()?;

	Utils::app(lua, &ya)?;
	Utils::cache(lua, &ya)?;
	Utils::call(lua, &ya)?;
	Utils::image(lua, &ya)?;
	Utils::layer(lua, &ya)?;
	Utils::log(lua, &ya)?;
	Utils::preview(lua, &ya)?;
	Utils::sync_isolate(lua, &ya)?;
	Utils::target(lua, &ya)?;
	Utils::text(lua, &ya)?;
	Utils::time(lua, &ya)?;
	Utils::user(lua, &ya)?;

	lua.globals().raw_set("ya", ya)
}
