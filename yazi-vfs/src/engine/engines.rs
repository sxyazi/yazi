use std::io;

use yazi_fs::{cha::Cha, engine::{Attrs, Capabilities as C, Engine, Transmit}, file::File};
use yazi_shared::{path::{DynPath, PathBufDyn}, strand::AsStrand, url::{Url, UrlBuf, UrlCow}};

pub(super) enum Engines<'a> {
	Local(yazi_fs::engine::local::Local<'a>),
	Lua(super::lua::Lua<'a>),
	Sftp(super::sftp::Sftp<'a>),
}

impl<'a> Engine for Engines<'a> {
	type Demand = super::Demand;
	type File = super::RwFile;
	type Me<'b> = Engines<'b>;
	type ReadDir = super::ReadDir;
	type UrlCow = UrlCow<'a>;

	async fn absolute(&self) -> io::Result<Self::UrlCow> { dispatch!(self, absolute) }

	async fn canonicalize(&self) -> io::Result<UrlBuf> { dispatch!(self, canonicalize) }

	async fn capabilities(&self) -> io::Result<C> {
		match self {
			Self::Local(p) => p.capabilities().await,
			Self::Lua(p) => p.capabilities().await,
			Self::Sftp(p) => p.capabilities().await,
		}
	}

	async fn casefold(&self) -> io::Result<UrlBuf> { dispatch!(self, casefold) }

	async fn copy_to(&self, to: Url<'_>, attrs: Attrs) -> io::Result<Transmit> {
		dispatch!(self, copy_to, to, attrs)
	}

	async fn copy_from(&self, from: Url<'_>, attrs: Attrs) -> io::Result<Transmit> {
		dispatch!(self, copy_from, from, attrs)
	}

	async fn create_dir(&self) -> io::Result<()> { dispatch!(self, create_dir) }

	async fn create_dir_all(&self) -> io::Result<()> { dispatch!(self, create_dir_all) }

	async fn file(&self) -> io::Result<File> { dispatch!(self, file) }

	async fn hard_link<P>(&self, to: P) -> io::Result<()>
	where
		P: DynPath,
	{
		dispatch!(self, hard_link, to)
	}

	async fn metadata(&self) -> io::Result<Cha> { dispatch!(self, metadata) }

	async fn new<'b>(url: Url<'b>) -> io::Result<Self::Me<'b>> {
		use yazi_shared::auth::AuthKind as K;

		Ok(match url.kind() {
			K::Regular => Self::Me::Local(yazi_fs::engine::local::Local::new(url).await?),
			K::Sftp => Self::Me::Sftp(super::sftp::Sftp::new(url).await?),
			K::Mount | K::Hub | K::Scope | K::View => Self::Me::Lua(super::lua::Lua::new(url).await?),
		})
	}

	async fn read_dir(self) -> io::Result<Self::ReadDir> {
		Ok(match self {
			Self::Local(p) => p.read_dir().await?.into(),
			Self::Lua(p) if p.handles(C::READ_DIR).await? => p.read_dir().await?.into(),
			Self::Lua(p) => physical!(p, read_dir)?,
			Self::Sftp(p) => p.read_dir().await?.into(),
		})
	}

	async fn read_link(&self) -> io::Result<PathBufDyn> { dispatch!(self, read_link) }

	async fn revalidate(&self, file: File) -> io::Result<Option<File>> {
		match self {
			Self::Local(p) => p.revalidate(file).await,
			Self::Lua(p) if p.handles(C::REVALIDATE).await? => p.revalidate(file).await,
			Self::Lua(p) => physical!(p, revalidate, File { url: file.url.into_physical(), ..file }),
			Self::Sftp(p) => p.revalidate(file).await,
		}
	}

	async fn remove_dir(&self) -> io::Result<()> { dispatch!(self, remove_dir) }

	async fn remove_dir_all(&self) -> io::Result<()> { dispatch!(self, remove_dir_all) }

	async fn remove_dir_clean(&self) -> io::Result<()> { dispatch!(self, remove_dir_clean) }

	async fn remove_file(&self) -> io::Result<()> { dispatch!(self, remove_file) }

	async fn rename<P>(&self, to: P) -> io::Result<()>
	where
		P: DynPath,
	{
		dispatch!(self, rename, to)
	}

	async fn set_attrs(&self, attrs: Attrs) -> io::Result<()> { dispatch!(self, set_attrs, attrs) }

	async fn symlink<S, F>(&self, original: S, is_dir: F) -> io::Result<()>
	where
		S: AsStrand,
		F: AsyncFnOnce() -> io::Result<bool>,
	{
		dispatch!(self, symlink, original, is_dir)
	}

	async fn symlink_dir<S>(&self, original: S) -> io::Result<()>
	where
		S: AsStrand,
	{
		dispatch!(self, symlink_dir, original)
	}

	async fn symlink_file<S>(&self, original: S) -> io::Result<()>
	where
		S: AsStrand,
	{
		dispatch!(self, symlink_file, original)
	}

	async fn symlink_metadata(&self) -> io::Result<Cha> { dispatch!(self, symlink_metadata) }

	async fn trash(&self) -> io::Result<()> { dispatch!(self, trash) }

	fn url(&self) -> Url<'_> {
		match self {
			Self::Local(p) => p.url(),
			Self::Lua(p) => p.url(),
			Self::Sftp(p) => p.url(),
		}
	}

	async fn write<B>(&self, contents: B) -> io::Result<()>
	where
		B: AsRef<[u8]>,
	{
		dispatch!(self, write, contents)
	}
}
