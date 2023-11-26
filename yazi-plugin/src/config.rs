use mlua::{Lua, LuaSerdeExt, SerializeOptions};
use yazi_config::{MANAGER, PREVIEW, THEME};

const OPTIONS: SerializeOptions =
	SerializeOptions::new().serialize_none_to_null(false).serialize_unit_to_null(false);

pub struct Config<'a> {
	lua: &'a Lua,
}

impl<'a> Config<'a> {
	pub fn new(lua: &'a Lua) -> Self { Self { lua } }

	pub fn install_theme(self) -> mlua::Result<Self> {
		self.lua.globals().set("THEME", self.lua.to_value_with(&*THEME, OPTIONS)?)?;
		Ok(self)
	}

	pub fn install_preview(self) -> mlua::Result<Self> {
		self.lua.globals().set("PREVIEW", self.lua.to_value_with(&*PREVIEW, OPTIONS)?)?;
		Ok(self)
	}

	pub fn install_manager(self) -> mlua::Result<Self> {
		self.lua.globals().set("MANAGER", self.lua.to_value_with(&*MANAGER, OPTIONS)?)?;
		Ok(self)
	}
}
