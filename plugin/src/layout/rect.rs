use mlua::{FromLua, Lua, UserData, Value};

#[derive(Clone, Copy)]
pub(crate) struct Rect(pub(crate) ratatui::layout::Rect);

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
		fields.add_field_method_get("width", |_, me| Ok(me.0.width));
		fields.add_field_method_get("height", |_, me| Ok(me.0.height));
	}
}
