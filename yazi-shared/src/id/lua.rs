use mlua::{AnyUserData, ExternalError, ExternalResult, FromLua, Lua, MetaMethod, UserData, UserDataFields, UserDataMethods, Value};
use yazi_shim::log::LOG_LEVEL;

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

	fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
		methods.add_meta_function(MetaMethod::Lt, |lua, (lhs, rhs): (Value, Value)| {
			Ok(Self::from_lua(lhs, lua)? < Self::from_lua(rhs, lua)?)
		});
		methods.add_meta_function(MetaMethod::Le, |lua, (lhs, rhs): (Value, Value)| {
			Ok(Self::from_lua(lhs, lua)? <= Self::from_lua(rhs, lua)?)
		});
		methods.add_meta_method(MetaMethod::ToString, |_, me, ()| Ok(me.to_string()));
		methods.add_meta_function(MetaMethod::Concat, |lua, (lhs, rhs): (Value, Value)| {
			match (lhs, rhs) {
				(Value::String(lhs), Value::UserData(rhs)) => {
					let rhs = rhs.borrow::<Self>()?;
					lua.create_external_string([&lhs.as_bytes(), rhs.to_string().as_bytes()].concat())
				}
				(Value::UserData(lhs), Value::String(rhs)) => {
					let lhs = lhs.borrow::<Self>()?;
					lua.create_external_string([lhs.to_string().as_bytes(), &rhs.as_bytes()].concat())
				}
				_ => Err("only string can be concatenated with Id".into_lua_err()),
			}
		});

		if !LOG_LEVEL.get().is_none() {
			methods.add_meta_function(MetaMethod::ToDebugString, |_, ud: AnyUserData| {
				Ok(format!("Id({:?}): {}", ud.to_pointer(), *ud.borrow::<Self>()?))
			});
		}
	}
}
