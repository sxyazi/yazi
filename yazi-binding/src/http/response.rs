use std::{io, mem, ops::Deref};

use futures::TryStreamExt;
use http_body_util::{BodyExt, StreamBody, combinators::BoxBody};
use hyper::{Response, body::{Body, Bytes, Frame, Incoming}};
use mlua::{BString, IntoLuaMulti, UserData, UserDataFields, UserDataMethods, UserDataRegistry, Value};
use tokio::io::{AsyncWrite, AsyncWriteExt};
use yazi_shim::{fs::Error, mlua::UserDataFieldsExt};

use super::HttpInventory;

pub struct HttpResponse {
	inner: Response<BoxBody<Bytes, io::Error>>,
	url:   String,
	len:   Option<u64>,
}

impl Deref for HttpResponse {
	type Target = Response<BoxBody<Bytes, io::Error>>;

	fn deref(&self) -> &Self::Target { &self.inner }
}

impl HttpResponse {
	pub(crate) fn new(mut response: reqwest::Response) -> Self {
		let url = response.url().as_str().to_owned();
		let len = response.content_length();
		let status = response.status();
		let headers = mem::take(response.headers_mut());

		let mut inner = Response::new(
			StreamBody::new(response.bytes_stream().map_ok(Frame::data).map_err(io::Error::other))
				.boxed(),
		);
		*inner.status_mut() = status;
		*inner.headers_mut() = headers;

		Self { inner, url, len }
	}

	pub(super) fn from_hyper(response: Response<Incoming>, url: String) -> Self {
		let len = response.body().size_hint().exact();
		Self { inner: response.map(|body| body.map_err(io::Error::other).boxed()), url, len }
	}

	pub async fn write<W: AsyncWrite + Unpin>(mut self, output: &mut W) -> io::Result<()> {
		while let Some(frame) = self.inner.body_mut().frame().await {
			if let Ok(data) = frame?.into_data() {
				output.write_all(&data).await?;
			}
		}
		Ok(())
	}

	async fn bytes(mut self) -> io::Result<Vec<u8>> {
		let mut bytes = Vec::new();
		while let Some(frame) = self.inner.body_mut().frame().await {
			if let Ok(data) = frame?.into_data() {
				bytes.extend_from_slice(&data);
			}
		}
		Ok(bytes)
	}
}

impl UserData for HttpResponse {
	fn register(registry: &mut UserDataRegistry<Self>) {
		Self::add_fields(registry);
		Self::add_methods(registry);

		for inv in inventory::iter::<HttpInventory> {
			(inv.register)(registry);
		}
	}

	fn add_fields<F: UserDataFields<Self>>(fields: &mut F) {
		fields.add_field_method_get("status", |_, me| Ok(me.status().as_u16()));
		fields.add_cached_field("url", |lua, me| lua.create_string(&me.url));
		fields.add_field_method_get("len", |_, me| Ok(me.len));
		fields.add_cached_field("headers", |lua, me| {
			lua.create_table_from(
				me.headers().iter().map(|(name, value)| (name.as_str(), BString::from(value.as_bytes()))),
			)
		});
	}

	fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
		methods.add_async_method_once("bytes", |lua, me, ()| async move {
			match me.bytes().await {
				Ok(bytes) => BString::from(bytes).into_lua_multi(&lua),
				Err(e) => (Value::Nil, Error::from(e)).into_lua_multi(&lua),
			}
		});
	}
}
