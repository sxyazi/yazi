use mlua::{AnyUserData, ExternalError, ExternalResult, FromLua, Lua, MetaMethod, UserData, UserDataFields, UserDataMethods, Value};

use crate::{LOG_LEVEL, id::Id};

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

	fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
		if !LOG_LEVEL.get().is_none() {
			methods.add_meta_function(MetaMethod::ToDebugString, |_, ud: AnyUserData| {
				Ok(format!("Id({:?}): {}", ud.to_pointer(), *ud.borrow::<Self>()?))
			});
		}
	}
}
