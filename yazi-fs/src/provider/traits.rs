use std::{borrow::Cow, ffi::OsStr, io, path::{Path, PathBuf}};

use tokio::io::{AsyncRead, AsyncWrite, AsyncWriteExt};
use yazi_macro::ok_or_not_found;

use crate::cha::Cha;

pub trait Provider {
	type File: AsyncRead + AsyncWrite + Unpin;
	type Gate: FileBuilder<File = Self::File>;
	type ReadDir: DirReader;

	fn cache<P>(_: P) -> Option<PathBuf>
	where
		P: AsRef<Path>;

	fn canonicalize<P>(path: P) -> impl Future<Output = io::Result<PathBuf>>
	where
		P: AsRef<Path>;

	fn copy<P, Q>(from: P, to: Q, cha: Cha) -> impl Future<Output = io::Result<u64>>
	where
		P: AsRef<Path>,
		Q: AsRef<Path>;

	fn create<P>(path: P) -> impl Future<Output = io::Result<Self::File>>
	where
		P: AsRef<Path>,
	{
		async move { Self::Gate::default().write(true).create(true).truncate(true).open(path).await }
	}

	fn create_dir<P>(path: P) -> impl Future<Output = io::Result<()>>
	where
		P: AsRef<Path>;

	fn create_dir_all<P>(path: P) -> impl Future<Output = io::Result<()>>
	where
		P: AsRef<Path>,
	{
		async move {
			let path = path.as_ref();
			if path == Path::new("") {
				return Ok(());
			}

			match Self::create_dir(path).await {
				Ok(()) => return Ok(()),
				Err(e) if e.kind() == io::ErrorKind::NotFound => {}
				Err(_) if Self::metadata(path).await.is_ok_and(|m| m.is_dir()) => return Ok(()),
				Err(e) => return Err(e),
			}
			match path.parent() {
				Some(p) => Self::create_dir_all(p).await?,
				None => return Err(io::Error::other("failed to create whole tree")),
			}
			match Self::create_dir(path).await {
				Ok(()) => Ok(()),
				Err(_) if Self::metadata(path).await.is_ok_and(|m| m.is_dir()) => Ok(()),
				Err(e) => Err(e),
			}
		}
	}

	fn hard_link<P, Q>(original: P, link: Q) -> impl Future<Output = io::Result<()>>
	where
		P: AsRef<Path>,
		Q: AsRef<Path>;

	fn metadata<P>(path: P) -> impl Future<Output = io::Result<std::fs::Metadata>>
	where
		P: AsRef<Path>;

	fn open<P>(path: P) -> impl Future<Output = io::Result<Self::File>>
	where
		P: AsRef<Path>,
	{
		async move { Self::Gate::default().read(true).open(path).await }
	}

	fn read_dir<P>(path: P) -> impl Future<Output = io::Result<Self::ReadDir>>
	where
		P: AsRef<Path>;

	fn read_link<P>(path: P) -> impl Future<Output = io::Result<PathBuf>>
	where
		P: AsRef<Path>;

	fn remove_dir<P>(path: P) -> impl Future<Output = io::Result<()>>
	where
		P: AsRef<Path>;

	fn remove_dir_all<P>(path: P) -> impl Future<Output = io::Result<()>>
	where
		P: AsRef<Path>,
	{
		async fn remove_dir_all_impl<S>(path: &Path) -> io::Result<()>
		where
			S: Provider + ?Sized,
		{
			let mut it = ok_or_not_found!(S::read_dir(path).await, return Ok(()));
			while let Some(child) = it.next().await? {
				let ft = ok_or_not_found!(child.file_type().await, continue);
				let result = if ft.is_dir() {
					remove_dir_all_impl::<S>(&child.path()).await
				} else {
					S::remove_file(&child.path()).await
				};

				() = ok_or_not_found!(result);
			}

			Ok(ok_or_not_found!(S::remove_dir(path).await))
		}

		async move {
			let path = path.as_ref();
			let ft = ok_or_not_found!(Self::symlink_metadata(path).await, return Ok(()));
			if ft.is_symlink() {
				Self::remove_file(path).await
			} else {
				remove_dir_all_impl::<Self>(path).await
			}
		}
	}

	fn remove_file<P>(path: P) -> impl Future<Output = io::Result<()>>
	where
		P: AsRef<Path>;

	fn rename<P, Q>(from: P, to: Q) -> impl Future<Output = io::Result<()>>
	where
		P: AsRef<Path>,
		Q: AsRef<Path>;

	fn symlink<P, Q, F>(original: P, link: Q, _is_dir: F) -> impl Future<Output = io::Result<()>>
	where
		P: AsRef<Path>,
		Q: AsRef<Path>,
		F: AsyncFnOnce() -> io::Result<bool>;

	fn symlink_dir<P, Q>(original: P, link: Q) -> impl Future<Output = io::Result<()>>
	where
		P: AsRef<Path>,
		Q: AsRef<Path>,
	{
		Self::symlink(original, link, async || Ok(true))
	}

	fn symlink_file<P, Q>(original: P, link: Q) -> impl Future<Output = io::Result<()>>
	where
		P: AsRef<Path>,
		Q: AsRef<Path>,
	{
		Self::symlink(original, link, async || Ok(false))
	}

	fn symlink_metadata<P>(path: P) -> impl Future<Output = io::Result<std::fs::Metadata>>
	where
		P: AsRef<Path>;

	fn trash<P>(path: P) -> impl Future<Output = io::Result<()>>
	where
		P: AsRef<Path>;

	fn write<P, C>(path: P, contents: C) -> impl Future<Output = io::Result<()>>
	where
		P: AsRef<Path>,
		C: AsRef<[u8]>,
	{
		async move { Self::create(path).await?.write_all(contents.as_ref()).await }
	}
}

// --- DirReader
pub trait DirReader {
	type Entry<'a>: FileHolder
	where
		Self: 'a;

	fn next(&mut self) -> impl Future<Output = io::Result<Option<Self::Entry<'_>>>>;
}

// --- FileHolder
pub trait FileHolder {
	fn path(&self) -> PathBuf;

	fn name(&self) -> Cow<'_, OsStr>;

	fn metadata(&self) -> impl Future<Output = io::Result<std::fs::Metadata>>;

	fn file_type(&self) -> impl Future<Output = io::Result<std::fs::FileType>>;
}

// --- FileOpener
pub trait FileBuilder: Default {
	type File: AsyncRead + AsyncWrite + Unpin;

	fn append(&mut self, append: bool) -> &mut Self;

	fn create(&mut self, create: bool) -> &mut Self;

	fn create_new(&mut self, create_new: bool) -> &mut Self;

	fn open<P>(&self, path: P) -> impl Future<Output = io::Result<Self::File>>
	where
		P: AsRef<Path>;

	fn read(&mut self, read: bool) -> &mut Self;

	fn truncate(&mut self, truncate: bool) -> &mut Self;

	fn write(&mut self, write: bool) -> &mut Self;
}
