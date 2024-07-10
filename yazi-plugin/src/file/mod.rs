#![allow(clippy::module_inception)]

mod file;

pub use file::*;

pub fn pour(lua: &mlua::Lua) -> mlua::Result<()> {
	file::File::register(lua)?;
	file::File::install(lua)?;

	Ok(())
}
