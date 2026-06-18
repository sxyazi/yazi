use mlua::{ExternalError, ExternalResult, FromLua, Lua, UserData, UserDataFields, Value};

use crate::id::Id;

impl FromLua for Id {
	fn from_lua(value: Value, _: &Lua) -> mlua::Result<Self> {
		Ok(match value {
			Value::Integer(i) => Self::try_from(i).into_lua_err()?,
			Value::UserData(ud) => *ud.borrow::<Self>()?,
			_ => Err("expected integer or userdata".into_lua_err())?,
		})
	}
}

impl UserData for Id {
	fn add_fields<F: UserDataFields<Self>>(fields: &mut F) {
		fields.add_field_method_get("value", |_, me| Ok(me.get()));
	}
}
