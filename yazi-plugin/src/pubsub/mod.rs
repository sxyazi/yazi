#![allow(clippy::module_inception)]

use mlua::Lua;

yazi_macro::mod_flat!(pubsub);

pub(super) fn install(lua: &'static Lua) -> mlua::Result<()> {
	Pubsub::install(lua)?;

	Ok(())
}
