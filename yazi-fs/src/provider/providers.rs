use std::{io, path::{Path, PathBuf}, sync::Arc};

use yazi_shared::{scheme::SchemeRef, url::{Url, UrlCow}};
use yazi_vfs::config::{ProviderSftp, Vfs};

use super::local::Local;
use crate::{cha::Cha, provider::Provider};

pub(super) struct Providers<'a>(Inner<'a>);

enum Inner<'a> {
	Regular,
	Search(Url<'a>),
	Sftp((super::sftp::Sftp, Url<'a>)),
}

impl<'a> Providers<'a> {
	pub(super) async fn new(url: Url<'a>) -> io::Result<Self> {
		Ok(match url.scheme {
			SchemeRef::Regular => Self(Inner::Regular),
			SchemeRef::Search(_) => Self(Inner::Search(url)),
			SchemeRef::Archive(_) => {
				Err(io::Error::new(io::ErrorKind::Unsupported, "Unsupported filesystem: archive"))?
			}
			SchemeRef::Sftp(name) => {
				Self(Inner::Sftp((Vfs::provider::<&ProviderSftp>(name).await?.into(), url)))
			}
		})
	}
}

impl Provider for Providers<'_> {
	type File = super::RwFile;
	type Gate = super::Gate;
	type ReadDir = super::ReadDir;

	async fn absolute<'a, U>(&self, url: U) -> io::Result<UrlCow<'a>>
	where
		U: Into<Url<'a>>,
	{
		match self.0 {
			Inner::Regular | Inner::Search(_) => Local.absolute(url).await,
			Inner::Sftp((p, _)) => p.absolute(url).await,
		}
	}

	async fn canonicalize<P>(&self, path: P) -> io::Result<PathBuf>
	where
		P: AsRef<Path>,
	{
		match self.0 {
			Inner::Regular | Inner::Search(_) => Local.canonicalize(path).await,
			Inner::Sftp((p, _)) => p.canonicalize(path).await,
		}
	}

	async fn casefold<P>(&self, path: P) -> io::Result<PathBuf>
	where
		P: AsRef<Path>,
	{
		match self.0 {
			Inner::Regular | Inner::Search(_) => Local.casefold(path).await,
			Inner::Sftp((p, _)) => p.casefold(path).await,
		}
	}

	async fn copy<P, Q>(&self, from: P, to: Q, cha: Cha) -> io::Result<u64>
	where
		P: AsRef<Path>,
		Q: AsRef<Path>,
	{
		match self.0 {
			Inner::Regular | Inner::Search(_) => Local.copy(from, to, cha).await,
			Inner::Sftp((p, _)) => p.copy(from, to, cha).await,
		}
	}

	async fn create<P>(&self, path: P) -> io::Result<Self::File>
	where
		P: AsRef<Path>,
	{
		Ok(match self.0 {
			Inner::Regular | Inner::Search(_) => Local.create(path).await?.into(),
			Inner::Sftp((p, _)) => p.create(path).await?.into(),
		})
	}

	async fn create_dir<P>(&self, path: P) -> io::Result<()>
	where
		P: AsRef<Path>,
	{
		match self.0 {
			Inner::Regular | Inner::Search(_) => Local.create_dir(path).await,
			Inner::Sftp((p, _)) => p.create_dir(path).await,
		}
	}

	async fn create_dir_all<P>(&self, path: P) -> io::Result<()>
	where
		P: AsRef<Path>,
	{
		match self.0 {
			Inner::Regular | Inner::Search(_) => Local.create_dir_all(path).await,
			Inner::Sftp((p, _)) => p.create_dir_all(path).await,
		}
	}

	async fn gate(&self) -> io::Result<Self::Gate> {
		Ok(match self.0 {
			Inner::Regular | Inner::Search(_) => Local.gate().await?.into(),
			Inner::Sftp((p, _)) => p.gate().await?.into(),
		})
	}

	async fn hard_link<P, Q>(&self, original: P, link: Q) -> io::Result<()>
	where
		P: AsRef<Path>,
		Q: AsRef<Path>,
	{
		match self.0 {
			Inner::Regular | Inner::Search(_) => Local.hard_link(original, link).await,
			Inner::Sftp((p, _)) => p.hard_link(original, link).await,
		}
	}

	async fn metadata<P>(&self, path: P) -> io::Result<Cha>
	where
		P: AsRef<Path>,
	{
		match self.0 {
			Inner::Regular | Inner::Search(_) => Local.metadata(path).await,
			Inner::Sftp((p, _)) => p.metadata(path).await,
		}
	}

	async fn open<P>(&self, path: P) -> io::Result<Self::File>
	where
		P: AsRef<Path>,
	{
		Ok(match self.0 {
			Inner::Regular | Inner::Search(_) => Local.open(path).await?.into(),
			Inner::Sftp((p, _)) => p.open(path).await?.into(),
		})
	}

	async fn read_dir<P>(&self, path: P) -> io::Result<Self::ReadDir>
	where
		P: AsRef<Path>,
	{
		Ok(match self.0 {
			Inner::Regular => Self::ReadDir::Regular(Local.read_dir(path).await?),
			Inner::Search(dir) => {
				Self::ReadDir::Search((Arc::new(dir.to_owned()), Local.read_dir(path).await?))
			}
			Inner::Sftp((p, dir)) => {
				Self::ReadDir::Sftp((Arc::new(dir.to_owned()), p.read_dir(path).await?))
			}
		})
	}

	async fn read_link<P>(&self, path: P) -> io::Result<PathBuf>
	where
		P: AsRef<Path>,
	{
		match self.0 {
			Inner::Regular | Inner::Search(_) => Local.read_link(path).await,
			Inner::Sftp((p, _)) => p.read_link(path).await,
		}
	}

	async fn remove_dir<P>(&self, path: P) -> io::Result<()>
	where
		P: AsRef<Path>,
	{
		match self.0 {
			Inner::Regular | Inner::Search(_) => Local.remove_dir(path).await,
			Inner::Sftp((p, _)) => p.remove_dir(path).await,
		}
	}

	async fn remove_dir_all<P>(&self, path: P) -> io::Result<()>
	where
		P: AsRef<Path>,
	{
		match self.0 {
			Inner::Regular | Inner::Search(_) => Local.remove_dir_all(path).await,
			Inner::Sftp((p, _)) => p.remove_dir_all(path).await,
		}
	}

	async fn remove_file<P>(&self, path: P) -> io::Result<()>
	where
		P: AsRef<Path>,
	{
		match self.0 {
			Inner::Regular | Inner::Search(_) => Local.remove_file(path).await,
			Inner::Sftp((p, _)) => p.remove_file(path).await,
		}
	}

	async fn rename<P, Q>(&self, from: P, to: Q) -> io::Result<()>
	where
		P: AsRef<Path>,
		Q: AsRef<Path>,
	{
		match self.0 {
			Inner::Regular | Inner::Search(_) => Local.rename(from, to).await,
			Inner::Sftp((p, _)) => p.rename(from, to).await,
		}
	}

	async fn symlink<P, Q, F>(&self, original: P, link: Q, is_dir: F) -> io::Result<()>
	where
		P: AsRef<Path>,
		Q: AsRef<Path>,
		F: AsyncFnOnce() -> io::Result<bool>,
	{
		match self.0 {
			Inner::Regular | Inner::Search(_) => Local.symlink(original, link, is_dir).await,
			Inner::Sftp((p, _)) => p.symlink(original, link, is_dir).await,
		}
	}

	async fn symlink_dir<P, Q>(&self, original: P, link: Q) -> io::Result<()>
	where
		P: AsRef<Path>,
		Q: AsRef<Path>,
	{
		match self.0 {
			Inner::Regular | Inner::Search(_) => Local.symlink_dir(original, link).await,
			Inner::Sftp((p, _)) => p.symlink_dir(original, link).await,
		}
	}

	async fn symlink_file<P, Q>(&self, original: P, link: Q) -> io::Result<()>
	where
		P: AsRef<Path>,
		Q: AsRef<Path>,
	{
		match self.0 {
			Inner::Regular | Inner::Search(_) => Local.symlink_file(original, link).await,
			Inner::Sftp((p, _)) => p.symlink_file(original, link).await,
		}
	}

	async fn symlink_metadata<P>(&self, path: P) -> io::Result<Cha>
	where
		P: AsRef<Path>,
	{
		match self.0 {
			Inner::Regular | Inner::Search(_) => Local.symlink_metadata(path).await,
			Inner::Sftp((p, _)) => p.symlink_metadata(path).await,
		}
	}

	async fn trash<P>(&self, path: P) -> io::Result<()>
	where
		P: AsRef<Path>,
	{
		match self.0 {
			Inner::Regular | Inner::Search(_) => Local.trash(path).await,
			Inner::Sftp((p, _)) => p.trash(path).await,
		}
	}

	async fn write<P, C>(&self, path: P, contents: C) -> io::Result<()>
	where
		P: AsRef<Path>,
		C: AsRef<[u8]>,
	{
		match self.0 {
			Inner::Regular | Inner::Search(_) => Local.write(path, contents).await,
			Inner::Sftp((p, _)) => p.write(path, contents).await,
		}
	}
}
