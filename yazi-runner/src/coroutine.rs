use mlua::{ExternalError, FromLua, FromLuaMulti, Function, Lua, MultiValue, Value};
use yazi_shim::fs::Error;

pub(crate) struct LuaCoroutine {
	next:   Function,
	values: Option<MultiValue>,
}

impl LuaCoroutine {
	pub(crate) async fn new(next: Function) -> mlua::Result<Self> {
		Ok(Self { values: Some(next.call_async(()).await?), next })
	}

	pub(crate) async fn next<T: FromLuaMulti>(&mut self, lua: &Lua) -> mlua::Result<Option<T>> {
		let mut values = match self.values.take() {
			Some(values) => values,
			None => self.next.call_async(true).await?,
		};

		if !values.front().is_none_or(Value::is_nil) {
			return T::from_lua_multi(values, lua).map(Some);
		}

		let _ = values.pop_front();
		if let Some(v) = values.pop_front().filter(|v| !v.is_nil()) {
			return Err(Error::from_lua(v, lua)?.into_lua_err());
		}

		Ok(None)
	}
}
