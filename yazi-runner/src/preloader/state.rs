use mlua::{FromLuaMulti, Lua, MultiValue};

#[derive(Default)]
pub struct PreloadState {
	pub complete: bool,
	pub error:    Option<yazi_binding::Error>,
}

impl FromLuaMulti for PreloadState {
	fn from_lua_multi(values: MultiValue, lua: &Lua) -> mlua::Result<Self> {
		let (complete, error) = FromLuaMulti::from_lua_multi(values, lua)?;
		Ok(Self { complete, error })
	}
}
