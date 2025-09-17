use std::{io, path::{Path, PathBuf}, time::UNIX_EPOCH};

use yazi_sftp::fs::{Attrs, Flags};

use crate::{cha::Cha, provider::Provider};

pub struct Sftp;

impl Provider for Sftp {
	type File = yazi_sftp::fs::File;
	type Gate = super::Gate;
	type ReadDir = super::ReadDir;

	fn cache<P>(_: P) -> Option<PathBuf>
	where
		P: AsRef<Path>,
	{
		todo!()
	}

	async fn canonicalize<P>(path: P) -> io::Result<PathBuf>
	where
		P: AsRef<Path>,
	{
		Ok(Self::op().await?.realpath(&path).await?)
	}

	async fn copy<P, Q>(from: P, to: Q, cha: Cha) -> io::Result<u64>
	where
		P: AsRef<Path>,
		Q: AsRef<Path>,
	{
		// FIXME: pull this out to a From<Cha> for Attrs impl
		let attrs = Attrs {
			size:     Some(cha.len),
			uid:      Some(cha.uid),
			gid:      Some(cha.gid),
			perm:     Some(cha.mode.bits() as _),
			atime:    cha
				.atime
				.and_then(|t| t.duration_since(UNIX_EPOCH).ok())
				.map(|d| d.as_secs() as u32),
			mtime:    cha
				.mtime
				.and_then(|t| t.duration_since(UNIX_EPOCH).ok())
				.map(|d| d.as_secs() as u32),
			extended: Default::default(),
		};

		let op = Self::op().await?;

		let mut from = op.open(&from, Flags::READ, Attrs::default()).await?;
		let mut to = op.open(&to, Flags::WRITE | Flags::CREATE | Flags::TRUNCATE, attrs).await?;

		tokio::io::copy(&mut from, &mut to).await
	}

	async fn create_dir<P>(path: P) -> io::Result<()>
	where
		P: AsRef<Path>,
	{
		Ok(Self::op().await?.mkdir(&path, Attrs::default()).await?)
	}

	async fn hard_link<P, Q>(original: P, link: Q) -> io::Result<()>
	where
		P: AsRef<Path>,
		Q: AsRef<Path>,
	{
		Ok(Self::op().await?.hardlink(&original, &link).await?)
	}

	async fn metadata<P>(path: P) -> io::Result<Cha>
	where
		P: AsRef<Path>,
	{
		todo!()
	}

	async fn read_dir<P>(path: P) -> io::Result<Self::ReadDir>
	where
		P: AsRef<Path>,
	{
		Ok(super::ReadDir(Self::op().await?.read_dir(&path).await?))
	}

	async fn read_link<P>(path: P) -> io::Result<PathBuf>
	where
		P: AsRef<Path>,
	{
		Ok(Self::op().await?.readlink(&path).await?)
	}

	async fn remove_dir<P>(path: P) -> io::Result<()>
	where
		P: AsRef<Path>,
	{
		Ok(Self::op().await?.rmdir(&path).await?)
	}

	async fn remove_file<P>(path: P) -> io::Result<()>
	where
		P: AsRef<Path>,
	{
		Ok(Self::op().await?.remove(&path).await?)
	}

	async fn rename<P, Q>(from: P, to: Q) -> io::Result<()>
	where
		P: AsRef<Path>,
		Q: AsRef<Path>,
	{
		Ok(Self::op().await?.rename(&from, &to).await?)
	}

	async fn symlink<P, Q, F>(original: P, link: Q, _is_dir: F) -> io::Result<()>
	where
		P: AsRef<Path>,
		Q: AsRef<Path>,
		F: AsyncFnOnce() -> io::Result<bool>,
	{
		Ok(Self::op().await?.symlink(&original, &link).await?)
	}

	async fn symlink_metadata<P>(path: P) -> io::Result<Cha>
	where
		P: AsRef<Path>,
	{
		todo!()
	}

	async fn trash<P>(_path: P) -> io::Result<()>
	where
		P: AsRef<Path>,
	{
		Err(io::Error::new(io::ErrorKind::Unsupported, "Trash not supported"))
	}
}

impl Sftp {
	pub(super) async fn op() -> io::Result<deadpool::managed::Object<Sftp>> {
		use deadpool::managed::PoolError;

		super::CONN.get().await.map_err(|e| match e {
			PoolError::Timeout(_) => io::Error::new(io::ErrorKind::TimedOut, e.to_string()),
			PoolError::Backend(e) => e,
			PoolError::Closed | PoolError::NoRuntimeSpecified | PoolError::PostCreateHook(_) => {
				io::Error::other(e.to_string())
			}
		})
	}
}

impl deadpool::managed::Manager for Sftp {
	type Error = io::Error;
	type Type = yazi_sftp::Operator;

	async fn create(&self) -> Result<Self::Type, Self::Error> { todo!() }

	async fn recycle(
		&self,
		obj: &mut Self::Type,
		metrics: &deadpool::managed::Metrics,
	) -> deadpool::managed::RecycleResult<Self::Error> {
		todo!()
	}

	fn detach(&self, _obj: &mut Self::Type) { todo!() }
}
