use mlua::{FromLua, Lua, Table, UserData, Value};

use crate::{GLOBALS, LUA};

#[derive(Clone, Copy)]
pub(crate) struct Rect(pub(crate) ratatui::layout::Rect);

impl Rect {
	pub(crate) fn install() -> mlua::Result<()> {
		let ui: Table = GLOBALS.get("ui")?;
		ui.set(
			"Rect",
			LUA.create_function(|_, args: Table| {
				Ok(Self(ratatui::layout::Rect {
					x:      args.get("x")?,
					y:      args.get("y")?,
					width:  args.get("w")?,
					height: args.get("h")?,
				}))
			})?,
		)
	}
}

impl<'lua> FromLua<'lua> for Rect {
	fn from_lua(value: Value<'lua>, _: &'lua Lua) -> mlua::Result<Self> {
		match value {
			Value::UserData(ud) => Ok(*ud.borrow::<Self>()?),
			_ => Err(mlua::Error::FromLuaConversionError {
				from:    value.type_name(),
				to:      "Rect",
				message: Some("expected a Rect".to_string()),
			}),
		}
	}
}

impl UserData for Rect {
	fn add_fields<'lua, F: mlua::UserDataFields<'lua, Self>>(fields: &mut F) {
		fields.add_field_method_get("x", |_, me| Ok(me.0.x));
		fields.add_field_method_get("y", |_, me| Ok(me.0.y));
		fields.add_field_method_get("w", |_, me| Ok(me.0.width));
		fields.add_field_method_get("h", |_, me| Ok(me.0.height));

		fields.add_field_method_get("left", |_, me| Ok(me.0.left()));
		fields.add_field_method_get("right", |_, me| Ok(me.0.right()));
		fields.add_field_method_get("top", |_, me| Ok(me.0.top()));
		fields.add_field_method_get("bottom", |_, me| Ok(me.0.bottom()));
	}
}
