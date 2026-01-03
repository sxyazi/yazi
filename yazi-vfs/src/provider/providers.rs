use std::io;

use tokio::sync::mpsc;
use yazi_fs::{cha::Cha, provider::{Attrs, Capabilities, Provider}};
use yazi_shared::{path::{AsPath, PathBufDyn}, strand::AsStrand, url::{Url, UrlBuf, UrlCow}};

#[derive(Clone)]
pub(super) enum Providers<'a> {
	Local(yazi_fs::provider::local::Local<'a>),
	Sftp(super::sftp::Sftp<'a>),
}

impl<'a> Provider for Providers<'a> {
	type File = super::RwFile;
	type Gate = super::Gate;
	type Me<'b> = Providers<'b>;
	type ReadDir = super::ReadDir;
	type UrlCow = UrlCow<'a>;

	async fn absolute(&self) -> io::Result<Self::UrlCow> {
		match self {
			Self::Local(p) => p.absolute().await,
			Self::Sftp(p) => p.absolute().await,
		}
	}

	async fn canonicalize(&self) -> io::Result<UrlBuf> {
		match self {
			Self::Local(p) => p.canonicalize().await,
			Self::Sftp(p) => p.canonicalize().await,
		}
	}

	fn capabilities(&self) -> Capabilities {
		match self {
			Self::Local(p) => p.capabilities(),
			Self::Sftp(p) => p.capabilities(),
		}
	}

	async fn casefold(&self) -> io::Result<UrlBuf> {
		match self {
			Self::Local(p) => p.casefold().await,
			Self::Sftp(p) => p.casefold().await,
		}
	}

	async fn copy<P>(&self, to: P, attrs: Attrs) -> io::Result<u64>
	where
		P: AsPath,
	{
		match self {
			Self::Local(p) => p.copy(to, attrs).await,
			Self::Sftp(p) => p.copy(to, attrs).await,
		}
	}

	fn copy_with_progress<P, A>(&self, to: P, attrs: A) -> io::Result<mpsc::Receiver<io::Result<u64>>>
	where
		P: AsPath,
		A: Into<Attrs>,
	{
		match self {
			Self::Local(p) => p.copy_with_progress(to, attrs),
			Self::Sftp(p) => p.copy_with_progress(to, attrs),
		}
	}

	async fn create(&self) -> io::Result<Self::File> {
		Ok(match self {
			Self::Local(p) => p.create().await?.into(),
			Self::Sftp(p) => p.create().await?.into(),
		})
	}

	async fn create_dir(&self) -> io::Result<()> {
		match self {
			Self::Local(p) => p.create_dir().await,
			Self::Sftp(p) => p.create_dir().await,
		}
	}

	async fn create_dir_all(&self) -> io::Result<()> {
		match self {
			Self::Local(p) => p.create_dir_all().await,
			Self::Sftp(p) => p.create_dir_all().await,
		}
	}

	async fn create_new(&self) -> io::Result<Self::File> {
		Ok(match self {
			Self::Local(p) => p.create_new().await?.into(),
			Self::Sftp(p) => p.create_new().await?.into(),
		})
	}

	async fn hard_link<P>(&self, to: P) -> io::Result<()>
	where
		P: AsPath,
	{
		match self {
			Self::Local(p) => p.hard_link(to).await,
			Self::Sftp(p) => p.hard_link(to).await,
		}
	}

	async fn metadata(&self) -> io::Result<Cha> {
		match self {
			Self::Local(p) => p.metadata().await,
			Self::Sftp(p) => p.metadata().await,
		}
	}

	async fn new<'b>(url: Url<'b>) -> io::Result<Self::Me<'b>> {
		use yazi_shared::scheme::SchemeKind as K;

		Ok(match url.kind() {
			K::Regular | K::Search => Self::Me::Local(yazi_fs::provider::local::Local::new(url).await?),
			K::Archive => {
				Err(io::Error::new(io::ErrorKind::Unsupported, "Unsupported filesystem: archive"))?
			}
			K::Sftp => Self::Me::Sftp(super::sftp::Sftp::new(url).await?),
		})
	}

	async fn open(&self) -> io::Result<Self::File> {
		Ok(match self {
			Self::Local(p) => p.open().await?.into(),
			Self::Sftp(p) => p.open().await?.into(),
		})
	}

	async fn read_dir(self) -> io::Result<Self::ReadDir> {
		Ok(match self {
			Self::Local(p) => Self::ReadDir::Local(p.read_dir().await?),
			Self::Sftp(p) => Self::ReadDir::Sftp(p.read_dir().await?),
		})
	}

	async fn read_link(&self) -> io::Result<PathBufDyn> {
		match self {
			Self::Local(p) => p.read_link().await,
			Self::Sftp(p) => p.read_link().await,
		}
	}

	async fn remove_dir(&self) -> io::Result<()> {
		match self {
			Self::Local(p) => p.remove_dir().await,
			Self::Sftp(p) => p.remove_dir().await,
		}
	}

	async fn remove_dir_all(&self) -> io::Result<()> {
		match self {
			Self::Local(p) => p.remove_dir_all().await,
			Self::Sftp(p) => p.remove_dir_all().await,
		}
	}

	async fn remove_file(&self) -> io::Result<()> {
		match self {
			Self::Local(p) => p.remove_file().await,
			Self::Sftp(p) => p.remove_file().await,
		}
	}

	async fn rename<P>(&self, to: P) -> io::Result<()>
	where
		P: AsPath,
	{
		match self {
			Self::Local(p) => p.rename(to).await,
			Self::Sftp(p) => p.rename(to).await,
		}
	}

	async fn symlink<S, F>(&self, original: S, is_dir: F) -> io::Result<()>
	where
		S: AsStrand,
		F: AsyncFnOnce() -> io::Result<bool>,
	{
		match self {
			Self::Local(p) => p.symlink(original, is_dir).await,
			Self::Sftp(p) => p.symlink(original, is_dir).await,
		}
	}

	async fn symlink_dir<S>(&self, original: S) -> io::Result<()>
	where
		S: AsStrand,
	{
		match self {
			Self::Local(p) => p.symlink_dir(original).await,
			Self::Sftp(p) => p.symlink_dir(original).await,
		}
	}

	async fn symlink_file<S>(&self, original: S) -> io::Result<()>
	where
		S: AsStrand,
	{
		match self {
			Self::Local(p) => p.symlink_file(original).await,
			Self::Sftp(p) => p.symlink_file(original).await,
		}
	}

	async fn symlink_metadata(&self) -> io::Result<Cha> {
		match self {
			Self::Local(p) => p.symlink_metadata().await,
			Self::Sftp(p) => p.symlink_metadata().await,
		}
	}

	async fn trash(&self) -> io::Result<()> {
		match self {
			Self::Local(p) => p.trash().await,
			Self::Sftp(p) => p.trash().await,
		}
	}

	fn url(&self) -> Url<'_> {
		match self {
			Self::Local(p) => p.url(),
			Self::Sftp(p) => p.url(),
		}
	}

	async fn write<C>(&self, contents: C) -> io::Result<()>
	where
		C: AsRef<[u8]>,
	{
		match self {
			Self::Local(p) => p.write(contents).await,
			Self::Sftp(p) => p.write(contents).await,
		}
	}
}
