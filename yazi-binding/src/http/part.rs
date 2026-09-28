use std::io;

use mlua::{IntoLuaMulti, UserData, UserDataMethods};
use reqwest::multipart::Part;
use tokio::sync::mpsc;
use yazi_codegen::FromLuaOwned;
use yazi_shim::fs::Error;

use super::HttpBody;

#[derive(FromLuaOwned)]
pub struct HttpPart {
	pub(super) name:         String,
	pub(super) filename:     Option<String>,
	pub(super) content_type: Option<String>,
	pub(super) rx:           mpsc::Receiver<Vec<u8>>,
}

impl TryFrom<HttpPart> for Part {
	type Error = reqwest::Error;

	fn try_from(value: HttpPart) -> Result<Self, Self::Error> {
		let mut part = Part::stream(HttpBody::Stream(value.rx));
		if let Some(filename) = value.filename {
			part = part.file_name(filename);
		}
		if let Some(content_type) = value.content_type {
			part = part.mime_str(&content_type)?;
		}
		Ok(part)
	}
}

impl HttpPart {
	pub fn new(
		name: String,
		filename: Option<String>,
		content_type: Option<String>,
	) -> (HttpPartTx, Self) {
		let (tx, rx) = mpsc::channel(1);
		(HttpPartTx(tx), Self { name, filename, content_type, rx })
	}
}

impl UserData for HttpPart {}

// --- HttpPartTx
pub struct HttpPartTx(mpsc::Sender<Vec<u8>>);

impl HttpPartTx {
	async fn write(&self, bytes: Vec<u8>) -> io::Result<()> {
		self.0.send(bytes).await.map_err(|_| io::ErrorKind::BrokenPipe.into())
	}

	async fn flush(&self) -> io::Result<()> {
		self.0.reserve().await.map(|_| ()).map_err(|_| io::ErrorKind::BrokenPipe.into())
	}
}

impl UserData for HttpPartTx {
	fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
		methods.add_async_method("write", |lua, me, bytes: Vec<u8>| async move {
			match me.write(bytes).await {
				Ok(()) => true.into_lua_multi(&lua),
				Err(e) => (false, Error::from(e)).into_lua_multi(&lua),
			}
		});
		methods.add_async_method("flush", |lua, me, ()| async move {
			match me.flush().await {
				Ok(()) => true.into_lua_multi(&lua),
				Err(e) => (false, Error::from(e)).into_lua_multi(&lua),
			}
		});
	}
}
