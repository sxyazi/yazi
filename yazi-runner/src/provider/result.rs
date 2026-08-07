use mlua::{FromLua, FromLuaMulti, Lua, MultiValue, Value};

pub struct ProvideResult<T>(pub Result<T, yazi_shim::fs::Error>);

impl<T> From<yazi_shim::fs::Error> for ProvideResult<T> {
	fn from(value: yazi_shim::fs::Error) -> Self { Self(Err(value)) }
}

impl<T> From<mlua::Error> for ProvideResult<T> {
	fn from(value: mlua::Error) -> Self { yazi_shim::fs::Error::other(value.to_string()).into() }
}

impl<T> From<tokio::task::JoinError> for ProvideResult<T> {
	fn from(value: tokio::task::JoinError) -> Self {
		yazi_shim::fs::Error::other(value.to_string()).into()
	}
}

impl<T: FromLua> FromLuaMulti for ProvideResult<T> {
	fn from_lua_multi(mut values: MultiValue, lua: &Lua) -> mlua::Result<Self> {
		let value = values.pop_front().unwrap_or(Value::Nil);
		let error = values.pop_front().unwrap_or(Value::Nil);

		Ok(Self(if error.is_nil() {
			T::from_lua(value, lua).map_err(|e| yazi_shim::fs::Error::other(e.to_string()))
		} else {
			Err(
				yazi_shim::fs::Error::from_lua(error, lua)
					.unwrap_or_else(|e| yazi_shim::fs::Error::other(e.to_string())),
			)
		}))
	}
}

impl ProvideResult<bool> {
	pub fn ok(self) -> Result<(), yazi_shim::fs::Error> {
		if self.0? {
			Ok(())
		} else {
			Err(yazi_shim::fs::Error::other("Lua VFS returned false without an Error"))
		}
	}
}
