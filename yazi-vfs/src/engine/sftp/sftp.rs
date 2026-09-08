use std::{io, sync::Arc};

use deadpool::managed::PoolError;
use yazi_config::vfs::{ServiceSftp, Vfs};
use yazi_fs::engine::{Capabilities, DirReader, Engine, FileHolder, Transmit};
use yazi_sftp::fs::Attrs;
use yazi_shared::{auth::AuthKind, path::{DynPath, PathBufDyn}, strand::AsStrand, url::{Url, UrlBuf, UrlCow, UrlLike}};

use super::Cha;
use crate::engine::sftp::Conn;

pub struct Sftp<'a> {
	url:             Url<'a>,
	pub(super) path: &'a typed_path::UnixPath,

	config: Arc<ServiceSftp>,
	pool:   deadpool::managed::Pool<Conn>,
}

impl<'a> Engine for Sftp<'a> {
	type Demand = super::Demand;
	type File = yazi_sftp::fs::File;
	type Me<'b> = Sftp<'b>;
	type ReadDir = super::ReadDir;
	type UrlCow = UrlCow<'a>;

	async fn absolute(&self) -> io::Result<Self::UrlCow> {
		Ok(if let Some(u) = crate::engine::try_absolute_impl(self.url) {
			u
		} else {
			self.canonicalize().await?.into()
		})
	}

	async fn canonicalize(&self) -> io::Result<UrlBuf> {
		Ok(UrlBuf::Unix {
			loc:  self.op().await?.realpath(self.path).await?.into(),
			auth: self.config.auth.clone(),
		})
	}

	async fn capabilities(&self) -> io::Result<Capabilities> {
		Ok(Capabilities::for_kind(AuthKind::Sftp))
	}

	async fn casefold(&self) -> io::Result<UrlBuf> {
		let Some((parent, name)) = self.url.parent().zip(self.url.name()) else {
			return Ok(self.url.to_owned());
		};

		if !self.symlink_metadata().await?.is_link() {
			return Ok(match self.canonicalize().await?.name() {
				Some(name) => parent.try_join(name)?,
				None => Err(io::Error::other("Cannot get filename"))?,
			});
		}

		let mut it = Self::new(parent).await?.read_dir().await?;
		let mut similar = None;
		while let Some(dent) = it.next().await? {
			let s = dent.name();
			if !name.eq_ignore_ascii_case(&s) {
				continue;
			} else if s == name {
				return Ok(dent.url());
			} else if similar.is_none() {
				similar = Some(s.into_owned());
			} else {
				return Err(io::ErrorKind::NotFound.into());
			}
		}

		similar.map(|n| parent.try_join(n)).transpose()?.ok_or(io::ErrorKind::NotFound.into())
	}

	async fn copy_to(&self, to: Url<'_>, attrs: yazi_fs::engine::Attrs) -> io::Result<Transmit> {
		let to = to.physical();
		if self.url.auth() != to.auth() {
			return Ok(Transmit::unsupported());
		}

		Ok(crate::engine::copy_progressive_impl(self.url.into(), to.into(), attrs))
	}

	async fn copy_from(&self, from: Url<'_>, attrs: yazi_fs::engine::Attrs) -> io::Result<Transmit> {
		let from = from.physical();
		if self.url.auth() != from.auth() {
			return Ok(Transmit::unsupported());
		}

		Ok(crate::engine::copy_progressive_impl(from.into(), self.url.into(), attrs))
	}

	async fn create_dir(&self) -> io::Result<()> {
		let op = self.op().await?;
		let result = op.mkdir(self.path, Attrs::default()).await;

		if let Err(yazi_sftp::Error::Status(status)) = &result
			&& status.is_failure()
			&& op.lstat(self.path).await.is_ok()
		{
			return Err(io::ErrorKind::AlreadyExists.into());
		}

		Ok(result?)
	}

	async fn hard_link<P>(&self, to: P) -> io::Result<()>
	where
		P: DynPath,
	{
		let to = to.dyn_path().as_unix()?;

		Ok(self.op().await?.hardlink(self.path, to).await?)
	}

	async fn metadata(&self) -> io::Result<yazi_fs::cha::Cha> {
		let attrs = self.op().await?.stat(self.path).await?;
		Ok(Cha::try_from((self.path.file_name().unwrap_or_default(), &attrs))?.0)
	}

	async fn new<'b>(url: Url<'b>) -> io::Result<Self::Me<'b>> {
		let Url::Unix { loc, auth } = url else {
			return Err(io::Error::new(io::ErrorKind::InvalidInput, format!("Not a SFTP URL: {url}")));
		};

		let config: Arc<ServiceSftp> = Vfs::service(auth)?;
		let pool = Conn::pool(config.clone());
		Ok(Self::Me { url, path: loc.as_inner(), config, pool })
	}

	async fn read_dir(self) -> io::Result<Self::ReadDir> {
		Ok(Self::ReadDir {
			dir:    Arc::new(self.url.to_owned()),
			reader: self.op().await?.read_dir(self.path).await?,
		})
	}

	async fn read_link(&self) -> io::Result<PathBufDyn> {
		Ok(self.op().await?.readlink(self.path).await?.into())
	}

	async fn remove_dir(&self) -> io::Result<()> { Ok(self.op().await?.rmdir(self.path).await?) }

	async fn remove_file(&self) -> io::Result<()> { Ok(self.op().await?.remove(self.path).await?) }

	async fn rename<P>(&self, to: P) -> io::Result<()>
	where
		P: DynPath,
	{
		let to = to.dyn_path().as_unix()?;
		let op = self.op().await?;

		match op.rename_posix(self.path, &to).await {
			Ok(()) => {}
			Err(yazi_sftp::Error::Unsupported) => {
				match op.remove(&to).await.map_err(io::Error::from) {
					Ok(()) => {}
					Err(e) if e.kind() == io::ErrorKind::NotFound => {}
					Err(e) => Err(e)?,
				}
				op.rename(self.path, &to).await?;
			}
			Err(e) => Err(e)?,
		}
		Ok(())
	}

	async fn set_attrs(&self, attrs: yazi_fs::engine::Attrs) -> io::Result<()> {
		let attrs = super::Attrs(attrs)
			.try_into()
			.map_err(|()| io::Error::new(io::ErrorKind::InvalidInput, "Cannot convert attributes"))?;

		Ok(self.op().await?.setstat(self.path, attrs).await?)
	}

	async fn symlink<S, F>(&self, original: S, _is_dir: F) -> io::Result<()>
	where
		S: AsStrand,
		F: AsyncFnOnce() -> io::Result<bool>,
	{
		let original = original.as_strand().encoded_bytes();

		Ok(self.op().await?.symlink(original, self.path).await?)
	}

	async fn symlink_metadata(&self) -> io::Result<yazi_fs::cha::Cha> {
		let attrs = self.op().await?.lstat(self.path).await?;
		Ok(Cha::try_from((self.path.file_name().unwrap_or_default(), &attrs))?.0)
	}

	async fn trash(&self) -> io::Result<()> {
		Err(io::Error::new(io::ErrorKind::Unsupported, "Trash not supported"))
	}

	#[inline]
	fn url(&self) -> Url<'_> { self.url }
}

impl<'a> Sftp<'a> {
	pub(super) async fn op(&self) -> io::Result<deadpool::managed::Object<Conn>> {
		self.pool.get().await.map_err(|e| match e {
			PoolError::Timeout(_) => io::Error::new(io::ErrorKind::TimedOut, e.to_string()),
			PoolError::Backend(e) => e,
			e => io::Error::other(e.to_string()),
		})
	}
}
