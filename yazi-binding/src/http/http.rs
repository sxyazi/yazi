use std::{io, ops::{Deref, DerefMut}};

use mlua::{BString, BorrowedBytes, ExternalResult, FromLua, Lua, LuaString, Table, UserData, UserDataFields, UserDataRegistry, Value};
use reqwest::{Method, header::{HeaderMap, HeaderName, HeaderValue}};
use tokio::io::{AsyncWrite, AsyncWriteExt};
use yazi_shim::mlua::UserDataFieldsExt;

use super::HttpInventory;

// --- HttpMethod
#[derive(Default)]
struct HttpMethod(Method);

impl FromLua for HttpMethod {
	fn from_lua(value: Value, lua: &Lua) -> mlua::Result<Self> {
		let b = BorrowedBytes::from_lua(value, lua)?;
		Ok(Self(Method::from_bytes(&b).into_lua_err()?))
	}
}

// --- HttpHeaders
#[derive(Default)]
struct HttpHeaders(HeaderMap);

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

// --- HttpRequest
pub struct HttpRequest {
	pub url:     String,
	pub method:  Method,
	pub headers: HeaderMap,
	pub body:    Option<Vec<u8>>,
}

impl FromLua for HttpRequest {
	fn from_lua(value: Value, lua: &Lua) -> mlua::Result<Self> {
		let t = Table::from_lua(value, lua)?;

		Ok(Self {
			url:     t.raw_get::<LuaString>("url")?.to_str()?.to_owned(),
			method:  t.raw_get::<Option<HttpMethod>>("method")?.unwrap_or_default().0,
			headers: t.raw_get::<Option<HttpHeaders>>("headers")?.unwrap_or_default().0,
			body:    t.raw_get::<Option<BorrowedBytes>>("body")?.map(|body| body.to_vec()),
		})
	}
}

// --- HttpResponse
pub struct HttpResponse(pub reqwest::Response);

impl Deref for HttpResponse {
	type Target = reqwest::Response;

	fn deref(&self) -> &Self::Target { &self.0 }
}

impl DerefMut for HttpResponse {
	fn deref_mut(&mut self) -> &mut Self::Target { &mut self.0 }
}

impl HttpResponse {
	pub async fn write<W: AsyncWrite + Unpin>(mut self, output: &mut W) -> io::Result<()> {
		while let Some(chunk) = self.chunk().await.map_err(io::Error::other)? {
			output.write_all(&chunk).await?;
		}
		Ok(())
	}
}

impl UserData for HttpResponse {
	fn register(registry: &mut UserDataRegistry<Self>) {
		Self::add_fields(registry);

		for inv in inventory::iter::<HttpInventory> {
			(inv.register)(registry);
		}
	}

	fn add_fields<F: UserDataFields<Self>>(fields: &mut F) {
		fields.add_field_method_get("status", |_, me| Ok(me.status().as_u16()));
		fields.add_cached_field("url", |lua, me| lua.create_string(me.url().as_str()));
		fields.add_field_method_get("length", |_, me| Ok(me.content_length()));
		fields.add_cached_field("headers", |lua, me| {
			lua.create_table_from(
				me.headers().iter().map(|(name, value)| (name.as_str(), BString::from(value.as_bytes()))),
			)
		});
	}
}
