use std::io;

use mlua::FromLua;
use tokio::sync::mpsc;
use yazi_binding::MpscTx;
use yazi_config::vfs::{ServiceLua, Vfs};
use yazi_fs::{cha::Cha, engine::{Attrs, Capabilities, Engine}, file::Files};
use yazi_runner::{RUNNER, provider::{ProvideResult, ProviderJob}};
use yazi_shared::{event::Cmd, path::{DynPath, PathBufDyn}, strand::AsStrand, url::{AsUrl, Url, UrlBuf, UrlCow}};

use crate::engine::lua::ReadDir;

#[derive(Clone)]
pub struct Lua<'a> {
	pub(super) url: Url<'a>,

	pub(super) run: &'static Cmd,
}

impl<'a> Engine for Lua<'a> {
	type Demand = super::Demand;
	type File = super::File;
	type Me<'b> = Lua<'b>;
	type ReadDir = ReadDir;
	type UrlCow = UrlCow<'static>;

	async fn absolute(&self) -> io::Result<Self::UrlCow> {
		let url = self.url.to_owned();

		Ok(self.call::<UrlBuf>(ProviderJob::Absolute { url }).await?.0?.into())
	}

	async fn canonicalize(&self) -> io::Result<UrlBuf> {
		let url = self.url.to_owned();

		Ok(self.call(ProviderJob::Canonicalize { url }).await?.0?)
	}

	async fn capabilities(&self) -> io::Result<Capabilities> {
		Ok(self.call(ProviderJob::Capabilities).await?.0?)
	}

	async fn casefold(&self) -> io::Result<UrlBuf> {
		let url = self.url.to_owned();

		Ok(self.call(ProviderJob::Casefold { url }).await?.0?)
	}

	async fn copy<P>(&self, to: P, attrs: Attrs) -> io::Result<u64>
	where
		P: DynPath,
	{
		let from = self.url.to_owned();
		let to = to.dyn_path().to_owned();

		Ok(self.call(ProviderJob::Copy { from, to, attrs }).await?.0?)
	}

	fn copy_progressive<P, A>(&self, to: P, attrs: A) -> io::Result<mpsc::Receiver<io::Result<u64>>>
	where
		P: DynPath,
		A: Into<Attrs>,
	{
		let (tx, rx) = mpsc::channel(20);
		let job = ProviderJob::CopyProgressive {
			from:  self.url.to_owned(),
			to:    to.dyn_path().to_owned(),
			attrs: attrs.into(),
			tx:    MpscTx::map(tx.clone(), Ok),
		};

		let run = self.run;
		tokio::spawn(async move {
			match RUNNER.provide(run, job).await.ok() {
				Ok(()) => tx.send(Ok(0)).await.ok(),
				Err(e) => tx.send(Err(e.into())).await.ok(),
			};
		});

		Ok(rx)
	}

	async fn create_dir(&self) -> io::Result<()> {
		let url = self.url.to_owned();

		Ok(self.call(ProviderJob::CreateDir { url }).await?.ok()?)
	}

	async fn hard_link<P>(&self, to: P) -> io::Result<()>
	where
		P: DynPath,
	{
		let from = self.url.to_owned();
		let to = to.dyn_path().to_owned();

		Ok(self.call(ProviderJob::HardLink { from, to }).await?.ok()?)
	}

	async fn metadata(&self) -> io::Result<Cha> {
		let url = self.url.to_owned();

		Ok(self.call(ProviderJob::Metadata { url }).await?.0?)
	}

	async fn new<'b>(url: Url<'b>) -> io::Result<Self::Me<'b>> {
		let (Url::Mount { auth, .. } | Url::Hub { auth, .. } | Url::Scope { auth, .. }) = url else {
			return Err(io::Error::new(
				io::ErrorKind::InvalidInput,
				format!("Not a custom VFS URL: {url:?}"),
			));
		};

		let service = Vfs::service::<&ServiceLua>(auth)?;
		Ok(Self::Me { url, run: &service.run })
	}

	async fn read_dir(self) -> io::Result<Self::ReadDir> {
		let url = self.url.to_owned();
		let files: Files = self.call(ProviderJob::ReadDir { url }).await?.0?;

		Ok(ReadDir { files: files.0.into_iter() })
	}

	async fn read_link(&self) -> io::Result<PathBufDyn> {
		let url = self.url.to_owned();

		Ok(self.call(ProviderJob::ReadLink { url }).await?.0?)
	}

	async fn remove_dir(&self) -> io::Result<()> {
		let url = self.url.to_owned();

		Ok(self.call(ProviderJob::RemoveDir { url }).await?.ok()?)
	}

	async fn remove_file(&self) -> io::Result<()> {
		let url = self.url.to_owned();

		Ok(self.call(ProviderJob::RemoveFile { url }).await?.ok()?)
	}

	async fn rename<P>(&self, to: P) -> io::Result<()>
	where
		P: DynPath,
	{
		let from = self.url.to_owned();
		let to = to.dyn_path().to_owned();

		Ok(self.call(ProviderJob::Rename { from, to }).await?.ok()?)
	}

	async fn set_attrs(&self, attrs: Attrs) -> io::Result<()> {
		let url = self.url.to_owned();

		Ok(self.call(ProviderJob::SetAttrs { url, attrs }).await?.ok()?)
	}

	async fn symlink<S, F>(&self, original: S, is_dir: F) -> io::Result<()>
	where
		S: AsStrand,
		F: AsyncFnOnce() -> io::Result<bool>,
	{
		let original = original.as_strand().encoded_bytes().to_vec();
		let url = self.url.to_owned();

		Ok(self.call(ProviderJob::Symlink { original, url, is_dir: is_dir().await? }).await?.ok()?)
	}

	async fn symlink_metadata(&self) -> io::Result<Cha> {
		let url = self.url.to_owned();

		Ok(self.call(ProviderJob::SymlinkMetadata { url }).await?.0?)
	}

	async fn trash(&self) -> io::Result<()> {
		let url = self.url.to_owned();

		Ok(self.call(ProviderJob::Trash { url }).await?.ok()?)
	}

	fn url(&self) -> Url<'_> { self.url.as_url() }

	async fn write<C>(&self, contents: C) -> io::Result<()>
	where
		C: AsRef<[u8]>,
	{
		let url = self.url.to_owned();
		let bytes = contents.as_ref().to_vec();

		Ok(self.call(ProviderJob::Write { url, offset: 0, bytes }).await?.ok()?)
	}
}

impl<'a> Lua<'a> {
	pub(super) async fn call<T>(&self, job: ProviderJob) -> io::Result<ProvideResult<T>>
	where
		T: FromLua + Send + 'static,
	{
		Ok(RUNNER.provide(self.run, job).await)
	}
}
