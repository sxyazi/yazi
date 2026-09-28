use std::io;

use mlua::{IntoLuaMulti, MetaMethod, UserData, UserDataMethods, Value};
use tokio::{sync::mpsc, task::JoinHandle};
use yazi_shim::fs::Error;

use super::HttpResponse;

pub(crate) struct HttpSession {
	pub(super) tx:     Option<mpsc::Sender<Vec<u8>>>,
	pub(super) handle: Option<JoinHandle<io::Result<HttpResponse>>>,
}

impl Drop for HttpSession {
	fn drop(&mut self) {
		if let Some(handle) = &self.handle {
			handle.abort();
		}
	}
}

impl HttpSession {
	async fn write(&self, bytes: Vec<u8>) -> io::Result<()> {
		self
			.tx
			.as_ref()
			.ok_or_else(|| io::Error::new(io::ErrorKind::Unsupported, "request has no streaming body"))?
			.send(bytes)
			.await
			.map_err(|_| io::ErrorKind::BrokenPipe.into())
	}

	async fn flush(&self) -> io::Result<()> {
		let Some(tx) = &self.tx else { return Ok(()) };
		tx.reserve().await.map(|_| ()).map_err(|_| io::ErrorKind::BrokenPipe.into())
	}

	async fn finish(&mut self) -> io::Result<HttpResponse> {
		self.tx = None;

		// Keep the handle here while waiting so __close can abort it on cancellation.
		let result = self.handle.as_mut().ok_or(io::ErrorKind::NotConnected)?.await;
		self.handle = None;

		result?
	}
}

impl UserData for HttpSession {
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
		methods.add_async_method_mut("finish", |lua, mut me, ()| async move {
			match me.finish().await {
				Ok(response) => response.into_lua_multi(&lua),
				Err(e) => (Value::Nil, Error::from(e)).into_lua_multi(&lua),
			}
		});
		methods.add_method("finished", |_, me, ()| {
			Ok(me.handle.as_ref().is_none_or(JoinHandle::is_finished))
		});
		methods.add_meta_method(MetaMethod::Close, |_, me, _: Value| {
			if let Some(handle) = &me.handle {
				handle.abort();
			}
			Ok(())
		});
	}
}
