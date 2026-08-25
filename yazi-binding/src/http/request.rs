use std::path::PathBuf;

use mlua::{BorrowedBytes, FromLua, Lua, LuaString, Table, Value};
use reqwest::{Method, header::HeaderMap};
use yazi_shared::path::PathBufDyn;

use super::{headers::HttpHeaders, method::HttpMethod};

pub struct HttpRequest {
	pub(crate) url:     String,
	pub(crate) method:  Method,
	pub(crate) headers: HeaderMap,
	pub(crate) body:    Option<Vec<u8>>,
	pub(crate) socket:  PathBuf,
}

impl FromLua for HttpRequest {
	fn from_lua(value: Value, lua: &Lua) -> mlua::Result<Self> {
		let t = Table::from_lua(value, lua)?;

		Ok(Self {
			url:     t.raw_get::<LuaString>("url")?.to_str()?.to_owned(),
			method:  t.raw_get::<Option<HttpMethod>>("method")?.unwrap_or_default().0,
			headers: t.raw_get::<Option<HttpHeaders>>("headers")?.unwrap_or_default().0,
			body:    t.raw_get::<Option<BorrowedBytes>>("body")?.map(|body| body.to_vec()),
			socket:  t
				.raw_get::<Option<PathBufDyn>>("socket")?
				.map(PathBufDyn::into_os)
				.transpose()?
				.unwrap_or_default(),
		})
	}
}
