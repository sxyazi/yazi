use mlua::{FromLua, FromLuaMulti, Lua, MultiValue, Value};

pub struct ProvideResult<T>(pub Result<T, yazi_binding::Error>);

impl<T> From<yazi_binding::Error> for ProvideResult<T> {
	fn from(value: yazi_binding::Error) -> Self { Self(Err(value)) }
}

impl<T> From<mlua::Error> for ProvideResult<T> {
	fn from(value: mlua::Error) -> Self { yazi_binding::Error::custom(value.to_string()).into() }
}

impl<T> From<tokio::task::JoinError> for ProvideResult<T> {
	fn from(value: tokio::task::JoinError) -> Self {
		yazi_binding::Error::custom(value.to_string()).into()
	}
}

impl<T: FromLua> FromLuaMulti for ProvideResult<T> {
	fn from_lua_multi(mut values: MultiValue, lua: &Lua) -> mlua::Result<Self> {
		let value = values.pop_front().unwrap_or(Value::Nil);
		let error = values.pop_front().unwrap_or(Value::Nil);

		Ok(Self(if error.is_nil() {
			T::from_lua(value, lua).map_err(|e| yazi_binding::Error::custom(e.to_string()))
		} else {
			Err(
				yazi_binding::Error::from_lua(error, lua)
					.unwrap_or_else(|e| yazi_binding::Error::custom(e.to_string())),
			)
		}))
	}
}

impl ProvideResult<bool> {
	pub fn ok(self) -> Result<(), yazi_binding::Error> {
		if self.0? {
			Ok(())
		} else {
			Err(yazi_binding::Error::custom("Lua VFS returned false without an Error"))
		}
	}
}
