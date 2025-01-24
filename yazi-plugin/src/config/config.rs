use mlua::{IntoLua, Lua, LuaSerdeExt, MetaMethod, SerializeOptions, Table, Value};
use yazi_boot::BOOT;
use yazi_config::{MANAGER, PREVIEW, THEME};

use super::Plugin;

pub const SER_OPTS: SerializeOptions =
	SerializeOptions::new().serialize_none_to_null(false).serialize_unit_to_null(false);

pub struct Config<'a> {
	lua: &'a Lua,
}

impl<'a> Config<'a> {
	pub fn new(lua: &'a Lua) -> Self { Self { lua } }

	pub fn install_boot(self) -> mlua::Result<Self> {
		self.lua.globals().raw_set("BOOT", self.lua.to_value_with(&*BOOT, SER_OPTS)?)?;
		Ok(self)
	}

	pub fn install_manager(self) -> mlua::Result<Self> {
		self.lua.globals().raw_set("MANAGER", self.lua.to_value_with(&*MANAGER, SER_OPTS)?)?;
		Ok(self)
	}

	pub fn install_theme(self) -> mlua::Result<Self> {
		self.lua.globals().raw_set("THEME", self.lua.to_value_with(&*THEME, SER_OPTS)?)?;
		Ok(self)
	}

	pub fn install_preview(self) -> mlua::Result<Self> {
		self.lua.globals().raw_set("PREVIEW", self.lua.to_value_with(&*PREVIEW, SER_OPTS)?)?;
		Ok(self)
	}

	pub fn install_plugin(self) -> mlua::Result<Self> {
		let index = self.lua.create_function(|lua, (ts, key): (Table, mlua::String)| {
			let value = match key.as_bytes().as_ref() {
				b"fetchers" => Plugin::fetchers(lua)?,
				b"spotter" => Plugin::spotter(lua)?,
				b"preloaders" => Plugin::preloaders(lua)?,
				b"previewer" => Plugin::previewer(lua)?,
				_ => return Ok(Value::Nil),
			}
			.into_lua(lua)?;

			ts.raw_set(key, value.clone())?;
			Ok(value)
		})?;

		let fetcher = self.lua.create_table()?;
		fetcher.set_metatable(Some(self.lua.create_table_from([(MetaMethod::Index.name(), index)])?));

		self.lua.globals().raw_set("PLUGIN", fetcher)?;
		Ok(self)
	}
}
