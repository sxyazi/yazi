use mlua::Lua;

pub fn install(lua: &Lua) -> mlua::Result<()> {
	super::Command::install(lua)?;

	Ok(())
}
