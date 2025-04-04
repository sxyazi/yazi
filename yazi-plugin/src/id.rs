use std::ops::Deref;

use mlua::{ExternalError, ExternalResult, FromLua, Lua, UserData, Value};

#[derive(Clone, Copy)]
pub struct Id(pub yazi_shared::Id);

impl Deref for Id {
	type Target = yazi_shared::Id;

	#[inline]
	fn deref(&self) -> &Self::Target { &self.0 }
}

impl FromLua for Id {
	fn from_lua(value: Value, _: &Lua) -> mlua::Result<Self> {
		Ok(match value {
			Value::Integer(i) => Self(i.try_into().into_lua_err()?),
			Value::UserData(ud) => *ud.borrow::<Self>()?,
			_ => Err("expected integer or userdata".into_lua_err())?,
		})
	}
}

impl UserData for Id {
	fn add_fields<F: mlua::UserDataFields<Self>>(fields: &mut F) {
		fields.add_field_method_get("value", |_, me| Ok(me.0.get()));
	}
}
