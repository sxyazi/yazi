use mlua::{BorrowedBytes, ExternalResult, FromLua, Lua, Table, Value};
use reqwest::header::{HeaderMap, HeaderName, HeaderValue};

#[derive(Default)]
pub(super) struct HttpHeaders(pub(super) HeaderMap);

impl FromLua for HttpHeaders {
	fn from_lua(value: Value, lua: &Lua) -> mlua::Result<Self> {
		let t = Table::from_lua(value, lua)?;
		let mut headers = HeaderMap::new();

		for pair in t.pairs::<BorrowedBytes, BorrowedBytes>() {
			let (name, value) = pair?;
			headers.insert(
				HeaderName::from_bytes(&name).into_lua_err()?,
				HeaderValue::from_bytes(&value).into_lua_err()?,
			);
		}
		Ok(Self(headers))
	}
}
