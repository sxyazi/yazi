use mlua::{BorrowedBytes, ExternalResult, FromLua, Lua, Value};
use reqwest::Method;

#[derive(Default)]
pub(super) struct HttpMethod(pub(super) Method);

impl FromLua for HttpMethod {
	fn from_lua(value: Value, lua: &Lua) -> mlua::Result<Self> {
		let b = BorrowedBytes::from_lua(value, lua)?;
		Ok(Self(Method::from_bytes(&b).into_lua_err()?))
	}
}
