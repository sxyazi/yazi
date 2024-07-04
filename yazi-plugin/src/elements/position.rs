use mlua::{AnyUserData, Lua, Table, UserDataFields, UserDataRef};

use crate::bindings::Cast;

pub type PositionRef<'lua> = UserDataRef<'lua, ratatui::layout::Position>;

pub struct Position;

impl Position {
	pub fn install(lua: &Lua, ui: &Table) -> mlua::Result<()> {
		let new = lua.create_function(|lua, (_, args): (Table, Table)| {
			Position::cast(lua, ratatui::layout::Position {
				x: args.raw_get("x")?,
				y: args.raw_get("y")?,
			})
		})?;

		let position =
			lua.create_table_from([("default", Position::cast(lua, Default::default())?)])?;

		position.set_metatable(Some(lua.create_table_from([("__call", new)])?));

		ui.raw_set("Position", position)
	}

	pub fn register(lua: &Lua) -> mlua::Result<()> {
		lua.register_userdata_type::<ratatui::layout::Position>(|reg| {
			reg.add_field_method_get("x", |_, me| Ok(me.x));
			reg.add_field_method_get("y", |_, me| Ok(me.y));
		})
	}
}

impl Cast<ratatui::layout::Position> for Position {
	fn cast(lua: &Lua, data: ratatui::layout::Position) -> mlua::Result<AnyUserData> {
		lua.create_any_userdata(data)
	}
}
