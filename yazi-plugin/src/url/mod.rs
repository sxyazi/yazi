#![allow(clippy::module_inception)]

mod url;

pub use url::*;

pub fn pour(lua: &mlua::Lua) -> mlua::Result<()> {
	url::Url::register(lua)?;
	url::Url::install(lua)?;

	Ok(())
}
