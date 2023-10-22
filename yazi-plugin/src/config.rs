use mlua::{LuaSerdeExt, SerializeOptions, Table};
use yazi_config::{MANAGER, THEME};

use crate::{layout::Rect, GLOBALS, LUA};

#[derive(Clone, Copy)]
pub(super) struct Config;

impl Config {
	pub(super) fn install(self) -> mlua::Result<()> {
		let options =
			SerializeOptions::new().serialize_none_to_null(false).serialize_unit_to_null(false);

		self.theme(options)?;
		self.manager(options)?;
		Ok(())
	}

	fn theme(self, options: SerializeOptions) -> mlua::Result<()> {
		GLOBALS.set("THEME", LUA.to_value_with(&*THEME, options)?)
	}

	fn manager(self, options: SerializeOptions) -> mlua::Result<()> {
		let manager = LUA.to_value_with(&*MANAGER, options)?;
		{
			let layout: Table = manager.as_table().unwrap().get("layout")?;

			layout.set(
				"preview_rect",
				LUA.create_function(|_, ()| Ok(Rect(MANAGER.layout.preview_rect())))?,
			)?;
			layout
				.set("preview_height", LUA.create_function(|_, ()| Ok(MANAGER.layout.preview_height()))?)?;
			layout
				.set("folder_rect", LUA.create_function(|_, ()| Ok(Rect(MANAGER.layout.folder_rect())))?)?;
			layout
				.set("folder_height", LUA.create_function(|_, ()| Ok(MANAGER.layout.folder_height()))?)?;
		}

		GLOBALS.set("MANAGER", manager)
	}
}
