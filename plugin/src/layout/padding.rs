use mlua::{FromLua, Lua, Table, UserData, Value};

use crate::{GLOBALS, LUA};

#[derive(Clone, Copy)]
pub(crate) struct Padding(pub(crate) ratatui::widgets::Padding);

impl Padding {
	pub(crate) fn install() -> mlua::Result<()> {
		let ui: Table = GLOBALS.get("ui")?;
		let padding: Table = ui.get("Padding")?;
		padding.set(
			"new",
			LUA.create_function(|_, args: (u16, u16, u16, u16)| {
				Ok(Self(ratatui::widgets::Padding::new(args.0, args.1, args.2, args.3)))
			})?,
		)
	}
}

impl<'lua> FromLua<'lua> for Padding {
	fn from_lua(value: Value<'lua>, _: &'lua Lua) -> mlua::Result<Self> {
		match value {
			Value::UserData(ud) => Ok(*ud.borrow::<Self>()?),
			_ => Err(mlua::Error::FromLuaConversionError {
				from:    value.type_name(),
				to:      "Padding",
				message: Some("expected a Padding".to_string()),
			}),
		}
	}
}

impl UserData for Padding {
	fn add_fields<'lua, F: mlua::UserDataFields<'lua, Self>>(fields: &mut F) {
		fields.add_field_method_get("left", |_, me| Ok(me.0.left));
		fields.add_field_method_get("right", |_, me| Ok(me.0.right));
		fields.add_field_method_get("top", |_, me| Ok(me.0.top));
		fields.add_field_method_get("bottom", |_, me| Ok(me.0.bottom));
	}
}
