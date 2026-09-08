use std::{io, sync::Arc};

use mlua::FromLua;
use tokio::sync::mpsc;
use yazi_config::vfs::{ServiceLua, Vfs};
use yazi_fs::{cha::Cha, engine::{Attrs, Capabilities as C, Engine, Transmit}, file::File};
use yazi_runner::{RUNNER, provider::{ProvideJob, ProvideResult}};
use yazi_shared::{path::{DynPath, PathBufDyn}, strand::AsStrand, url::{AsUrl, Url, UrlBuf, UrlCow}};

use crate::engine::lua::ReadDir;

pub struct Lua<'a> {
	pub(crate) url:     Url<'a>,
	pub(crate) service: Arc<ServiceLua>,
}

impl<'a> Engine for Lua<'a> {
	type Demand = super::Demand;
	type File = super::File;
	type Me<'b> = Lua<'b>;
	type ReadDir = ReadDir;
	type UrlCow = UrlCow<'static>;

	async fn absolute(&self) -> io::Result<Self::UrlCow> {
		let url = self.url.to_owned();

		Ok(self.call::<UrlBuf>(ProvideJob::Absolute { url }).await.0?.into())
	}

	async fn canonicalize(&self) -> io::Result<UrlBuf> {
		let url = self.url.to_owned();

		Ok(self.call(ProvideJob::Canonicalize { url }).await.0?)
	}

	async fn capabilities(&self) -> io::Result<C> {
		self
			.service
			.caps
			.get_or_try_init(|| async { Ok(self.call(ProvideJob::Capabilities).await.0?) })
			.await
			.copied()
	}

	async fn casefold(&self) -> io::Result<UrlBuf> {
		let url = self.url.to_owned();

		Ok(self.call(ProvideJob::Casefold { url }).await.0?)
	}

	async fn copy_to(&self, to: Url<'_>, attrs: Attrs) -> io::Result<Transmit> {
		if !self.capabilities().await?.contains(C::COPY_TO) {
			return Ok(Transmit::unsupported());
		}

		let (tx, rx) = mpsc::channel(20);
		tokio::spawn(RUNNER.provide_stream(
			self.service.clone(),
			ProvideJob::CopyTo { from: self.url.into(), to: to.into(), attrs },
			tx,
		));

		Ok(rx.into())
	}

	async fn copy_from(&self, from: Url<'_>, attrs: Attrs) -> io::Result<Transmit> {
		if !self.capabilities().await?.contains(C::COPY_FROM) {
			return Ok(Transmit::unsupported());
		}

		let (tx, rx) = mpsc::channel(20);
		tokio::spawn(RUNNER.provide_stream(
			self.service.clone(),
			ProvideJob::CopyFrom { from: from.into(), to: self.url.into(), attrs },
			tx,
		));

		Ok(rx.into())
	}

	async fn create_dir(&self) -> io::Result<()> {
		let url = self.url.to_owned();

		Ok(self.call(ProvideJob::CreateDir { url }).await.ok()?)
	}

	async fn create_dir_all(&self) -> io::Result<()> {
		if self.capabilities().await?.contains(C::CREATE_DIR_ALL) {
			let url = self.url.to_owned();
			Ok(self.call(ProvideJob::CreateDirAll { url }).await.ok()?)
		} else {
			self.create_dir_all_default().await
		}
	}

	async fn file(&self) -> io::Result<File> {
		let url = self.url.to_owned();

		Ok(self.call(ProvideJob::File { url }).await.0?)
	}

	async fn hard_link<P>(&self, to: P) -> io::Result<()>
	where
		P: DynPath,
	{
		let from = self.url.to_owned();
		let to = to.dyn_path().to_owned();

		Ok(self.call(ProvideJob::HardLink { from, to }).await.ok()?)
	}

	async fn metadata(&self) -> io::Result<Cha> {
		let url = self.url.to_owned();

		Ok(self.call(ProvideJob::Metadata { url }).await.0?)
	}

	async fn new<'b>(url: Url<'b>) -> io::Result<Self::Me<'b>> {
		Ok(Self::Me { url, service: Vfs::service(url.auth())? })
	}

	async fn read_dir(self) -> io::Result<Self::ReadDir> {
		let url = self.url.to_owned();
		let (tx, rx) = mpsc::channel(200);

		tokio::spawn(RUNNER.provide_stream(self.service, ProvideJob::ReadDir { url }, tx));
		Ok(ReadDir(rx))
	}

	async fn read_link(&self) -> io::Result<PathBufDyn> {
		let url = self.url.to_owned();

		Ok(self.call(ProvideJob::ReadLink { url }).await.0?)
	}

	async fn revalidate(&self, file: File) -> io::Result<Option<File>> {
		Ok(self.call(ProvideJob::Revalidate { file }).await.0?)
	}

	async fn remove_dir(&self) -> io::Result<()> {
		let url = self.url.to_owned();

		Ok(self.call(ProvideJob::RemoveDir { url }).await.ok()?)
	}

	async fn remove_dir_all(&self) -> io::Result<()> {
		if self.capabilities().await?.contains(C::REMOVE_DIR_ALL) {
			let url = self.url.to_owned();
			Ok(self.call(ProvideJob::RemoveDirAll { url }).await.ok()?)
		} else {
			self.remove_dir_all_default().await
		}
	}

	async fn remove_file(&self) -> io::Result<()> {
		let url = self.url.to_owned();

		Ok(self.call(ProvideJob::RemoveFile { url }).await.ok()?)
	}

	async fn rename<P>(&self, to: P) -> io::Result<()>
	where
		P: DynPath,
	{
		let from = self.url.to_owned();
		let to = to.dyn_path().to_owned();

		Ok(self.call(ProvideJob::Rename { from, to }).await.ok()?)
	}

	async fn set_attrs(&self, attrs: Attrs) -> io::Result<()> {
		let url = self.url.to_owned();

		Ok(self.call(ProvideJob::SetAttrs { url, attrs }).await.ok()?)
	}

	async fn symlink<S, F>(&self, original: S, is_dir: F) -> io::Result<()>
	where
		S: AsStrand,
		F: AsyncFnOnce() -> io::Result<bool>,
	{
		let original = original.as_strand().encoded_bytes().to_vec();
		let url = self.url.to_owned();

		Ok(self.call(ProvideJob::Symlink { original, url, is_dir: is_dir().await? }).await.ok()?)
	}

	async fn symlink_metadata(&self) -> io::Result<Cha> {
		let url = self.url.to_owned();

		Ok(self.call(ProvideJob::SymlinkMetadata { url }).await.0?)
	}

	async fn trash(&self) -> io::Result<()> {
		let url = self.url.to_owned();

		Ok(self.call(ProvideJob::Trash { url }).await.ok()?)
	}

	fn url(&self) -> Url<'_> { self.url.as_url() }

	async fn write<C>(&self, contents: C) -> io::Result<()>
	where
		C: AsRef<[u8]>,
	{
		let url = self.url.to_owned();
		let bytes = contents.as_ref().to_vec();

		Ok(self.call(ProvideJob::Write { url, offset: 0, bytes }).await.ok()?)
	}
}

impl<'a> Lua<'a> {
	pub(super) async fn call<T>(&self, job: ProvideJob) -> ProvideResult<T>
	where
		T: FromLua + Send + 'static,
	{
		RUNNER.provide(self.service.clone(), job).await
	}

	pub(crate) async fn handles(&self, caps: C) -> io::Result<bool> {
		Ok(!self.url.is_view() || self.capabilities().await?.contains(caps))
	}
}
