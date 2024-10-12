#![allow(clippy::module_inception)]

yazi_macro::mod_flat!(pubsub);

pub(super) fn install(lua: &'static mlua::Lua) -> mlua::Result<()> {
	Pubsub::install(lua)?;

	Ok(())
}
