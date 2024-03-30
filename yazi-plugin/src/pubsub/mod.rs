#![allow(clippy::module_inception)]

mod pubsub;

pub use pubsub::*;

pub(super) fn install(lua: &'static mlua::Lua) -> mlua::Result<()> {
	Pubsub::install(lua)?;

	Ok(())
}
