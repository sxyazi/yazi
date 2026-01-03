use std::{io, sync::Arc};

use tokio::{io::{AsyncWriteExt, BufReader, BufWriter}, sync::mpsc::Receiver};
use yazi_config::vfs::{ServiceSftp, Vfs};
use yazi_fs::provider::{Capabilities, DirReader, FileHolder, Provider};
use yazi_sftp::fs::{Attrs, Flags};
use yazi_shared::{loc::LocBuf, path::{AsPath, PathBufDyn}, pool::InternStr, scheme::SchemeKind, strand::AsStrand, url::{Url, UrlBuf, UrlCow, UrlLike}};

use super::Cha;
use crate::provider::sftp::Conn;

#[derive(Clone)]
pub struct Sftp<'a> {
	url:  Url<'a>,
	path: &'a typed_path::UnixPath,

	name:   &'static str,
	config: &'static ServiceSftp,
}

impl<'a> Provider for Sftp<'a> {
	type File = yazi_sftp::fs::File;
	type Gate = super::Gate;
	type Me<'b> = Sftp<'b>;
	type ReadDir = super::ReadDir;
	type UrlCow = UrlCow<'a>;

	async fn absolute(&self) -> io::Result<Self::UrlCow> {
		Ok(if let Some(u) = super::try_absolute(self.url) {
			u
		} else {
			self.canonicalize().await?.into()
		})
	}

	async fn canonicalize(&self) -> io::Result<UrlBuf> {
		Ok(UrlBuf::Sftp {
			loc:    self.op().await?.realpath(self.path).await?.into(),
			domain: self.name.intern(),
		})
	}

	fn capabilities(&self) -> Capabilities { Capabilities { symlink: true } }

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
		while let Some(entry) = it.next().await? {
			let s = entry.name();
			if !name.eq_ignore_ascii_case(&s) {
				continue;
			} else if s == name {
				return Ok(entry.url());
			} else if similar.is_none() {
				similar = Some(s.into_owned());
			} else {
				return Err(io::Error::from(io::ErrorKind::NotFound));
			}
		}

		similar
			.map(|n| parent.try_join(n))
			.transpose()?
			.ok_or_else(|| io::Error::from(io::ErrorKind::NotFound))
	}

	async fn copy<P>(&self, to: P, attrs: yazi_fs::provider::Attrs) -> io::Result<u64>
	where
		P: AsPath,
	{
		let to = to.as_path().as_unix()?;
		let attrs = super::Attrs(attrs).try_into().unwrap_or_default();

		let op = self.op().await?;
		let from = op.open(self.path, Flags::READ, &Attrs::default()).await?;
		let to = op.open(to, Flags::WRITE | Flags::CREATE | Flags::TRUNCATE, &attrs).await?;

		let mut reader = BufReader::with_capacity(524288, from);
		let mut writer = BufWriter::with_capacity(524288, to);
		let written = tokio::io::copy(&mut reader, &mut writer).await?;

		writer.flush().await?;
		if !attrs.is_empty() {
			writer.get_ref().fsetstat(&attrs).await.ok();
		}

		writer.shutdown().await.ok();
		Ok(written)
	}

	fn copy_with_progress<P, A>(&self, to: P, attrs: A) -> io::Result<Receiver<io::Result<u64>>>
	where
		P: AsPath,
		A: Into<yazi_fs::provider::Attrs>,
	{
		let to = UrlBuf::Sftp {
			loc:    LocBuf::<typed_path::UnixPathBuf>::saturated(
				to.as_path().to_unix_owned()?,
				SchemeKind::Sftp,
			),
			domain: self.name.intern(),
		};
		let from = self.url.to_owned();

		Ok(crate::provider::copy_with_progress_impl(from, to, attrs.into()))
	}

	async fn create_dir(&self) -> io::Result<()> {
		let op = self.op().await?;
		let result = op.mkdir(self.path, Attrs::default()).await;

		if let Err(yazi_sftp::Error::Status(status)) = &result
			&& status.is_failure()
			&& op.lstat(self.path).await.is_ok()
		{
			return Err(io::Error::from(io::ErrorKind::AlreadyExists));
		}

		Ok(result?)
	}

	async fn hard_link<P>(&self, to: P) -> io::Result<()>
	where
		P: AsPath,
	{
		let to = to.as_path().as_unix()?;

		Ok(self.op().await?.hardlink(self.path, to).await?)
	}

	async fn metadata(&self) -> io::Result<yazi_fs::cha::Cha> {
		let attrs = self.op().await?.stat(self.path).await?;
		Ok(Cha::try_from((self.path.file_name().unwrap_or_default(), &attrs))?.0)
	}

	async fn new<'b>(url: Url<'b>) -> io::Result<Self::Me<'b>> {
		match url {
			Url::Regular(_) | Url::Search { .. } | Url::Archive { .. } => {
				Err(io::Error::new(io::ErrorKind::InvalidInput, format!("Not a SFTP URL: {url:?}")))
			}
			Url::Sftp { loc, domain } => {
				let (name, config) = Vfs::service::<&ServiceSftp>(domain).await?;
				Ok(Self::Me { url, path: loc.as_inner(), name, config })
			}
		}
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
		P: AsPath,
	{
		let to = to.as_path().as_unix()?;
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
	#[inline]
	pub(super) async fn op(&self) -> io::Result<deadpool::managed::Object<Conn>> {
		Conn { name: self.name, config: self.config }.roll().await
	}
}
