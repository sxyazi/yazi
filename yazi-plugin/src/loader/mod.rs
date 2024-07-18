#![allow(clippy::module_inception)]

mod loader;
mod require;

pub use loader::*;
use require::*;

pub(super) fn init() { LOADER.with(<_>::default); }

pub(super) fn install(lua: &'static mlua::Lua) -> mlua::Result<()> {
	Require::install(lua)?;

	Ok(())
}
