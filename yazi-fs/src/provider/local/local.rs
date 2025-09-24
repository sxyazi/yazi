use std::{io, path::{Path, PathBuf}};

use yazi_shared::url::{Url, UrlCow};

use crate::{cha::Cha, path::absolute_url, provider::Provider};

#[derive(Clone, Copy)]
pub struct Local;

impl Provider for Local {
	type File = tokio::fs::File;
	type Gate = super::Gate;
	type ReadDir = super::ReadDir;

	async fn absolute<'a, U>(&self, url: U) -> io::Result<UrlCow<'a>>
	where
		U: Into<Url<'a>>,
	{
		let url: Url = url.into();
		if url.scheme.is_virtual() {
			Err(io::Error::new(io::ErrorKind::InvalidInput, "Not a local URL"))
		} else {
			Ok(absolute_url(url))
		}
	}

	#[inline]
	async fn canonicalize<P>(&self, path: P) -> io::Result<PathBuf>
	where
		P: AsRef<Path>,
	{
		tokio::fs::canonicalize(path).await
	}

	async fn casefold<P>(&self, path: P) -> io::Result<PathBuf>
	where
		P: AsRef<Path>,
	{
		super::casefold(path).await
	}

	#[inline]
	async fn copy<P, Q>(&self, from: P, to: Q, cha: Cha) -> io::Result<u64>
	where
		P: AsRef<Path>,
		Q: AsRef<Path>,
	{
		let from = from.as_ref().to_owned();
		let to = to.as_ref().to_owned();
		Self::copy_impl(from, to, cha).await
	}

	#[inline]
	async fn create_dir<P>(&self, path: P) -> io::Result<()>
	where
		P: AsRef<Path>,
	{
		tokio::fs::create_dir(path).await
	}

	#[inline]
	async fn create_dir_all<P>(&self, path: P) -> io::Result<()>
	where
		P: AsRef<Path>,
	{
		tokio::fs::create_dir_all(path).await
	}

	#[inline]
	async fn gate(&self) -> io::Result<Self::Gate> { Ok(Self::Gate::default()) }

	#[inline]
	async fn hard_link<P, Q>(&self, original: P, link: Q) -> io::Result<()>
	where
		P: AsRef<Path>,
		Q: AsRef<Path>,
	{
		tokio::fs::hard_link(original, link).await
	}

	#[inline]
	async fn metadata<P>(&self, path: P) -> io::Result<Cha>
	where
		P: AsRef<Path>,
	{
		let path = path.as_ref();
		Ok(Cha::new(path.file_name().unwrap_or_default(), tokio::fs::metadata(path).await?))
	}

	#[inline]
	async fn read_dir<P>(&self, path: P) -> io::Result<Self::ReadDir>
	where
		P: AsRef<Path>,
	{
		tokio::fs::read_dir(path).await.map(super::ReadDir)
	}

	#[inline]
	async fn read_link<P>(&self, path: P) -> io::Result<PathBuf>
	where
		P: AsRef<Path>,
	{
		tokio::fs::read_link(path).await
	}

	#[inline]
	async fn remove_dir<P>(&self, path: P) -> io::Result<()>
	where
		P: AsRef<Path>,
	{
		tokio::fs::remove_dir(path).await
	}

	#[inline]
	async fn remove_dir_all<P>(&self, path: P) -> io::Result<()>
	where
		P: AsRef<Path>,
	{
		tokio::fs::remove_dir_all(path).await
	}

	#[inline]
	async fn remove_file<P>(&self, path: P) -> io::Result<()>
	where
		P: AsRef<Path>,
	{
		tokio::fs::remove_file(path).await
	}

	#[inline]
	async fn rename<P, Q>(&self, from: P, to: Q) -> io::Result<()>
	where
		P: AsRef<Path>,
		Q: AsRef<Path>,
	{
		tokio::fs::rename(from, to).await
	}

	#[inline]
	async fn symlink<P, Q, F>(&self, original: P, link: Q, _is_dir: F) -> io::Result<()>
	where
		P: AsRef<Path>,
		Q: AsRef<Path>,
		F: AsyncFnOnce() -> io::Result<bool>,
	{
		#[cfg(unix)]
		{
			tokio::fs::symlink(original, link).await
		}
		#[cfg(windows)]
		if _is_dir().await? {
			self.symlink_dir(original, link).await
		} else {
			self.symlink_file(original, link).await
		}
	}

	#[inline]
	async fn symlink_dir<P, Q>(&self, original: P, link: Q) -> io::Result<()>
	where
		P: AsRef<Path>,
		Q: AsRef<Path>,
	{
		#[cfg(unix)]
		{
			tokio::fs::symlink(original, link).await
		}
		#[cfg(windows)]
		{
			tokio::fs::symlink_dir(original, link).await
		}
	}

	#[inline]
	async fn symlink_file<P, Q>(&self, original: P, link: Q) -> io::Result<()>
	where
		P: AsRef<Path>,
		Q: AsRef<Path>,
	{
		#[cfg(unix)]
		{
			tokio::fs::symlink(original, link).await
		}
		#[cfg(windows)]
		{
			tokio::fs::symlink_file(original, link).await
		}
	}

	#[inline]
	async fn symlink_metadata<P>(&self, path: P) -> io::Result<Cha>
	where
		P: AsRef<Path>,
	{
		let path = path.as_ref();
		Ok(Cha::new(path.file_name().unwrap_or_default(), tokio::fs::symlink_metadata(path).await?))
	}

	async fn trash<P>(&self, path: P) -> io::Result<()>
	where
		P: AsRef<Path>,
	{
		let path = path.as_ref().to_owned();
		tokio::task::spawn_blocking(move || {
			#[cfg(target_os = "android")]
			{
				Err(io::Error::new(io::ErrorKind::Unsupported, "Unsupported OS for trash operation"))
			}
			#[cfg(target_os = "macos")]
			{
				use trash::{TrashContext, macos::{DeleteMethod, TrashContextExtMacos}};
				let mut ctx = TrashContext::default();
				ctx.set_delete_method(DeleteMethod::NsFileManager);
				ctx.delete(path).map_err(io::Error::other)
			}
			#[cfg(all(not(target_os = "macos"), not(target_os = "android")))]
			{
				trash::delete(path).map_err(io::Error::other)
			}
		})
		.await?
	}

	#[inline]
	async fn write<P, C>(&self, path: P, contents: C) -> io::Result<()>
	where
		P: AsRef<Path>,
		C: AsRef<[u8]>,
	{
		tokio::fs::write(path, contents).await
	}
}

impl Local {
	async fn copy_impl(from: PathBuf, to: PathBuf, cha: Cha) -> io::Result<u64> {
		#[cfg(any(target_os = "linux", target_os = "android"))]
		{
			use std::os::unix::fs::OpenOptionsExt;

			tokio::task::spawn_blocking(move || {
				let mut reader = std::fs::File::open(from)?;
				let mut writer = std::fs::OpenOptions::new()
					.mode(cha.mode.bits() as _)
					.write(true)
					.create(true)
					.truncate(true)
					.open(to)?;

				let written = std::io::copy(&mut reader, &mut writer)?;
				writer.set_permissions(cha.into()).ok();
				writer.set_times(cha.into()).ok();

				Ok(written)
			})
			.await?
		}

		#[cfg(not(any(target_os = "linux", target_os = "android")))]
		{
			tokio::task::spawn_blocking(move || {
				let written = std::fs::copy(from, &to)?;

				if let Ok(file) = std::fs::File::options().write(true).open(to) {
					file.set_times(cha.into()).ok();
				}

				Ok(written)
			})
			.await?
		}
	}

	#[inline]
	pub async fn read<P>(&self, path: P) -> io::Result<Vec<u8>>
	where
		P: AsRef<Path>,
	{
		tokio::fs::read(path).await
	}

	#[inline]
	pub async fn read_to_string<P>(&self, path: P) -> io::Result<String>
	where
		P: AsRef<Path>,
	{
		tokio::fs::read_to_string(path).await
	}
}
