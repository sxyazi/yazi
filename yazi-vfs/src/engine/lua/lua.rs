use std::{io, sync::Arc};

use mlua::FromLua;
use tokio::sync::mpsc;
use yazi_config::vfs::{ServiceLua, Vfs};
use yazi_fs::{engine::{Attrs, Capabilities, Engine, Transmit}, file::File, stat::Stat};
use yazi_runner::{RUNNER, provider::{ProvideJob, ProvideOp, ProvidePeer, ProvideResult}};
use yazi_shared::{path::{DynPath, PathBufDyn}, strand::AsStrand, url::{AsUrl, Url, UrlBuf, UrlCow, UrlLike}};

use crate::engine::lua::ReadDir;

pub struct Lua<'a> {
	pub(crate) url:     Url<'a>,
	pub(crate) service: Arc<ServiceLua>,
}

impl AsUrl for Lua<'_> {
	fn as_url(&self) -> Url<'_> { self.url }
}

impl UrlLike for Lua<'_> {}

impl<'a> Engine for Lua<'a> {
	type Demand = super::Demand;
	type File = super::File;
	type Me<'b> = Lua<'b>;
	type ReadDir = ReadDir;
	type UrlCow = UrlCow<'static>;

	async fn absolute(&self) -> io::Result<Self::UrlCow> {
		let url = self.to_url();

		Ok(self.call::<UrlBuf>(ProvideOp::Absolute { url }).await.0?.into())
	}

	async fn canonicalize(&self) -> io::Result<UrlBuf> {
		let url = self.to_url();

		Ok(self.call(ProvideOp::Canonicalize { url }).await.0?)
	}

	async fn capabilities(&self) -> io::Result<Capabilities> {
		self
			.service
			.caps
			.get_or_try_init(|| async { Ok(self.call(ProvideOp::Capabilities).await.0?) })
			.await
			.copied()
	}

	async fn casefold(&self) -> io::Result<UrlBuf> {
		let url = self.to_url();

		Ok(self.call(ProvideOp::Casefold { url }).await.0?)
	}

	async fn copy_to(&self, to: Url<'_>, attrs: Attrs) -> io::Result<Transmit> {
		if !self.capabilities().await?.copy_to {
			return Ok(Transmit::unsupported());
		}

		let peer = ProvidePeer::new(to)?;
		let (tx, rx) = mpsc::channel(20);
		tokio::spawn(RUNNER.provide_stream(
			ProvideJob::new(&self.service, ProvideOp::CopyTo {
				from: self.to_url(),
				to: to.into(),
				peer,
				attrs,
			}),
			tx,
		));

		Ok(rx.into())
	}

	async fn copy_from(&self, from: Url<'_>, attrs: Attrs) -> io::Result<Transmit> {
		if !self.capabilities().await?.copy_from {
			return Ok(Transmit::unsupported());
		}

		let peer = ProvidePeer::new(from)?;
		let (tx, rx) = mpsc::channel(20);
		tokio::spawn(RUNNER.provide_stream(
			ProvideJob::new(&self.service, ProvideOp::CopyFrom {
				from: from.into(),
				to: self.to_url(),
				peer,
				attrs,
			}),
			tx,
		));

		Ok(rx.into())
	}

	async fn create_dir(&self) -> io::Result<()> {
		let url = self.to_url();

		Ok(self.call(ProvideOp::CreateDir { url }).await.ok()?)
	}

	async fn create_dir_all(&self) -> io::Result<()> {
		if self.capabilities().await?.create_dir_all {
			let url = self.to_url();
			Ok(self.call(ProvideOp::CreateDirAll { url }).await.ok()?)
		} else {
			self.create_dir_all_default().await
		}
	}

	async fn create_file(&self) -> io::Result<()> {
		let url = self.to_url();

		Ok(self.call(ProvideOp::CreateFile { url }).await.ok()?)
	}

	async fn create_file_new(&self) -> io::Result<()> {
		let url = self.to_url();

		Ok(self.call(ProvideOp::CreateFileNew { url }).await.ok()?)
	}

	async fn file(&self) -> io::Result<File> {
		let url = self.to_url();

		Ok(self.call(ProvideOp::File { url, handle: None }).await.0?)
	}

	async fn hard_link<P>(&self, to: P) -> io::Result<()>
	where
		P: DynPath,
	{
		let from = self.to_url();
		let to = to.dyn_path().to_owned();

		Ok(self.call(ProvideOp::HardLink { from, to }).await.ok()?)
	}

	async fn metadata(&self) -> io::Result<Stat> {
		let url = self.to_url();

		Ok(self.call(ProvideOp::Metadata { url, handle: None }).await.0?)
	}

	async fn new<'b>(url: Url<'b>) -> io::Result<Self::Me<'b>> {
		Ok(Self::Me { url, service: Vfs::service(url.auth())? })
	}

	async fn read_dir(self) -> io::Result<Self::ReadDir> {
		let url = self.to_url();
		let (tx, rx) = mpsc::channel(200);

		let job = ProvideJob { service: self.service, op: ProvideOp::ReadDir { url } };
		tokio::spawn(RUNNER.provide_stream(job, tx));

		Ok(ReadDir(rx))
	}

	async fn read_link(&self) -> io::Result<PathBufDyn> {
		let url = self.to_url();

		Ok(self.call(ProvideOp::ReadLink { url }).await.0?)
	}

	async fn reroute(&self) -> io::Result<File> {
		let cap = self.capabilities().await?.reroute;
		let mask = if self.is_absolute() { 0b10 } else { 0b01 };
		if cap & mask == 0 {
			return Err(io::ErrorKind::Unsupported.into());
		}

		let url = self.to_url();
		Ok(self.call(ProvideOp::Reroute { url }).await.0?)
	}

	async fn revalidate(&self, file: File) -> io::Result<Option<File>> {
		Ok(self.call(ProvideOp::Revalidate { file }).await.0?)
	}

	async fn remove_dir(&self) -> io::Result<()> {
		let url = self.to_url();

		Ok(self.call(ProvideOp::RemoveDir { url }).await.ok()?)
	}

	async fn remove_dir_all(&self) -> io::Result<()> {
		if self.capabilities().await?.remove_dir_all {
			let url = self.to_url();
			Ok(self.call(ProvideOp::RemoveDirAll { url }).await.ok()?)
		} else {
			self.remove_dir_all_default().await
		}
	}

	async fn remove_file(&self) -> io::Result<()> {
		let url = self.to_url();

		Ok(self.call(ProvideOp::RemoveFile { url }).await.ok()?)
	}

	async fn rename<P>(&self, to: P) -> io::Result<()>
	where
		P: DynPath,
	{
		let from = self.to_url();
		let to = to.dyn_path().to_owned();

		Ok(self.call(ProvideOp::Rename { from, to }).await.ok()?)
	}

	async fn set_attrs(&self, attrs: Attrs) -> io::Result<()> {
		let url = self.to_url();

		Ok(self.call(ProvideOp::SetAttrs { url, attrs, handle: None }).await.ok()?)
	}

	async fn symlink<S, F>(&self, original: S, is_dir: F) -> io::Result<()>
	where
		S: AsStrand,
		F: AsyncFnOnce() -> io::Result<bool>,
	{
		let original = original.as_strand().encoded_bytes().to_vec();
		let url = self.to_url();

		Ok(self.call(ProvideOp::Symlink { original, url, is_dir: is_dir().await? }).await.ok()?)
	}

	async fn symlink_metadata(&self) -> io::Result<Stat> {
		let url = self.to_url();

		Ok(self.call(ProvideOp::SymlinkMetadata { url }).await.0?)
	}

	async fn trash(&self) -> io::Result<()> {
		let url = self.to_url();

		Ok(self.call(ProvideOp::Trash { url }).await.ok()?)
	}
}

impl<'a> Lua<'a> {
	pub(super) async fn call<T>(&self, op: ProvideOp) -> ProvideResult<T>
	where
		T: FromLua + Send + 'static,
	{
		RUNNER.provide(ProvideJob::new(&self.service, op)).await
	}

	pub(crate) async fn handles(&self, check: fn(Capabilities) -> bool) -> io::Result<bool> {
		Ok(!self.is_view() || check(self.capabilities().await?))
	}
}
