use std::{io, ops::Deref, sync::Arc};

use mlua::FromLua;
use tokio::{sync::mpsc, task::JoinHandle};
use yazi_config::vfs::ServiceLua;
use yazi_runner::{RUNNER, provider::{Handle, ProvideJob, ProvideOp}};
use yazi_shared::url::{AsUrl, Url, UrlBuf, UrlLike};

pub struct FileHandle {
	pub(super) inner:   Handle,
	pub(super) url:     UrlBuf,
	pub(super) service: Arc<ServiceLua>,
}

impl Deref for FileHandle {
	type Target = Handle;

	fn deref(&self) -> &Self::Target { &self.inner }
}

impl Drop for FileHandle {
	fn drop(&mut self) {
		let op = ProvideOp::Close { url: self.to_url(), handle: self.inner.clone() };
		let job = ProvideJob::new(&self.service, op);

		tokio::spawn(async move {
			let _ = RUNNER.provide(job).await.ok();
		});
	}
}

impl AsUrl for FileHandle {
	fn as_url(&self) -> Url<'_> { self.url.as_url() }
}

impl UrlLike for FileHandle {}

impl FileHandle {
	pub(super) fn new(inner: Handle, url: impl Into<UrlBuf>, service: Arc<ServiceLua>) -> Self {
		Self { inner, url: url.into(), service }
	}

	pub(super) fn stream<T>(
		self: &Arc<Self>,
		op: ProvideOp,
		tx: mpsc::Sender<io::Result<T>>,
	) -> JoinHandle<()>
	where
		T: FromLua + Send + 'static,
	{
		let handle = self.clone();

		tokio::spawn(async move {
			RUNNER.provide_stream(ProvideJob::new(&handle.service, op), tx).await;
			// A stream can outlive its File; keep the resource open until the provider stops.
			drop(handle);
		})
	}
}
