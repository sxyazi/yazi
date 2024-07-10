#![allow(clippy::module_inception)]

mod cha;

pub use cha::*;

pub fn pour(lua: &mlua::Lua) -> mlua::Result<()> {
	cha::Cha::register(lua)?;
	cha::Cha::install(lua)?;

	Ok(())
}
