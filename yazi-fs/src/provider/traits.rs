use std::{borrow::Cow, ffi::OsStr, io, path::{Path, PathBuf}};

use tokio::io::{AsyncRead, AsyncWrite, AsyncWriteExt};
use yazi_macro::ok_or_not_found;
use yazi_shared::{scheme::SchemeRef, url::{Url, UrlCow}};

use crate::cha::{Cha, ChaType};

pub trait Provider {
	type File: AsyncRead + AsyncWrite + Unpin;
	type Gate: FileBuilder<File = Self::File>;
	type ReadDir: DirReader;

	fn absolute<'a, U>(&self, url: U) -> impl Future<Output = io::Result<UrlCow<'a>>>
	where
		U: Into<Url<'a>>;

	fn canonicalize<P>(&self, path: P) -> impl Future<Output = io::Result<PathBuf>>
	where
		P: AsRef<Path>;

	fn casefold<P>(&self, path: P) -> impl Future<Output = io::Result<PathBuf>>
	where
		P: AsRef<Path>;

	fn copy<P, Q>(&self, from: P, to: Q, cha: Cha) -> impl Future<Output = io::Result<u64>>
	where
		P: AsRef<Path>,
		Q: AsRef<Path>;

	fn create<P>(&self, path: P) -> impl Future<Output = io::Result<Self::File>>
	where
		P: AsRef<Path>,
	{
		async move { self.gate().await?.write(true).create(true).truncate(true).open(path).await }
	}

	fn create_dir<P>(&self, path: P) -> impl Future<Output = io::Result<()>>
	where
		P: AsRef<Path>;

	fn create_dir_all<P>(&self, path: P) -> impl Future<Output = io::Result<()>>
	where
		P: AsRef<Path>,
	{
		async move {
			let mut path = path.as_ref();
			if path == Path::new("") {
				return Ok(());
			}

			let mut stack = Vec::new();
			loop {
				match self.create_dir(path).await {
					Ok(()) => break,
					Err(e) if e.kind() == io::ErrorKind::NotFound => {
						if let Some(parent) = path.parent() {
							stack.push(path);
							path = parent;
						} else {
							return Err(io::Error::other("failed to create whole tree"));
						}
					}
					Err(_) if self.metadata(path).await.is_ok_and(|m| m.is_dir()) => break,
					Err(e) => return Err(e),
				}
			}

			while let Some(p) = stack.pop() {
				match self.create_dir(p).await {
					Ok(()) => {}
					Err(_) if self.metadata(p).await.is_ok_and(|m| m.is_dir()) => {}
					Err(e) => return Err(e),
				}
			}

			Ok(())
		}
	}

	fn gate(&self) -> impl Future<Output = io::Result<Self::Gate>>;

	fn hard_link<P, Q>(&self, original: P, link: Q) -> impl Future<Output = io::Result<()>>
	where
		P: AsRef<Path>,
		Q: AsRef<Path>;

	fn metadata<P>(&self, path: P) -> impl Future<Output = io::Result<Cha>>
	where
		P: AsRef<Path>;

	fn open<P>(&self, path: P) -> impl Future<Output = io::Result<Self::File>>
	where
		P: AsRef<Path>,
	{
		async move { self.gate().await?.read(true).open(path).await }
	}

	fn read_dir<P>(&self, path: P) -> impl Future<Output = io::Result<Self::ReadDir>>
	where
		P: AsRef<Path>;

	fn read_link<P>(&self, path: P) -> impl Future<Output = io::Result<PathBuf>>
	where
		P: AsRef<Path>;

	fn remove_dir<P>(&self, path: P) -> impl Future<Output = io::Result<()>>
	where
		P: AsRef<Path>;

	fn remove_dir_all<P>(&self, path: P) -> impl Future<Output = io::Result<()>>
	where
		P: AsRef<Path>,
	{
		async fn remove_dir_all_impl<P>(me: &P, path: &Path) -> io::Result<()>
		where
			P: Provider + ?Sized,
		{
			let mut it = ok_or_not_found!(me.read_dir(path).await, return Ok(()));
			while let Some(child) = it.next().await? {
				let ft = ok_or_not_found!(child.file_type().await, continue);
				let result = if ft.is_dir() {
					Box::pin(remove_dir_all_impl(me, &child.path())).await
				} else {
					me.remove_file(&child.path()).await
				};

				() = ok_or_not_found!(result);
			}

			Ok(ok_or_not_found!(me.remove_dir(path).await))
		}

		async move {
			let path = path.as_ref();
			let cha = ok_or_not_found!(self.symlink_metadata(path).await, return Ok(()));
			if cha.is_link() {
				self.remove_file(path).await
			} else {
				remove_dir_all_impl(self, path).await
			}
		}
	}

	fn remove_file<P>(&self, path: P) -> impl Future<Output = io::Result<()>>
	where
		P: AsRef<Path>;

	fn rename<P, Q>(&self, from: P, to: Q) -> impl Future<Output = io::Result<()>>
	where
		P: AsRef<Path>,
		Q: AsRef<Path>;

	fn symlink<P, Q, F>(
		&self,
		original: P,
		link: Q,
		_is_dir: F,
	) -> impl Future<Output = io::Result<()>>
	where
		P: AsRef<Path>,
		Q: AsRef<Path>,
		F: AsyncFnOnce() -> io::Result<bool>;

	fn symlink_dir<P, Q>(&self, original: P, link: Q) -> impl Future<Output = io::Result<()>>
	where
		P: AsRef<Path>,
		Q: AsRef<Path>,
	{
		self.symlink(original, link, async || Ok(true))
	}

	fn symlink_file<P, Q>(&self, original: P, link: Q) -> impl Future<Output = io::Result<()>>
	where
		P: AsRef<Path>,
		Q: AsRef<Path>,
	{
		self.symlink(original, link, async || Ok(false))
	}

	fn symlink_metadata<P>(&self, path: P) -> impl Future<Output = io::Result<Cha>>
	where
		P: AsRef<Path>;

	fn trash<P>(&self, path: P) -> impl Future<Output = io::Result<()>>
	where
		P: AsRef<Path>;

	fn write<P, C>(&self, path: P, contents: C) -> impl Future<Output = io::Result<()>>
	where
		P: AsRef<Path>,
		C: AsRef<[u8]>,
	{
		async move { self.create(path).await?.write_all(contents.as_ref()).await }
	}
}

// --- DirReader
pub trait DirReader {
	type Entry: FileHolder;

	fn next(&mut self) -> impl Future<Output = io::Result<Option<Self::Entry>>>;
}

// --- FileHolder
pub trait FileHolder {
	#[must_use]
	fn path(&self) -> PathBuf;

	#[must_use]
	fn name(&self) -> Cow<'_, OsStr>;

	fn metadata(&self) -> impl Future<Output = io::Result<Cha>>;

	fn file_type(&self) -> impl Future<Output = io::Result<ChaType>>;
}

// --- FileOpener
pub trait FileBuilder {
	type File: AsyncRead + AsyncWrite + Unpin;

	fn append(&mut self, append: bool) -> &mut Self;

	fn cha(&mut self, cha: Cha) -> &mut Self;

	fn create(&mut self, create: bool) -> &mut Self;

	fn create_new(&mut self, create_new: bool) -> &mut Self;

	fn new(scheme: SchemeRef) -> impl Future<Output = io::Result<Self>>
	where
		Self: Sized;

	fn open<P>(&self, path: P) -> impl Future<Output = io::Result<Self::File>>
	where
		P: AsRef<Path>;

	fn read(&mut self, read: bool) -> &mut Self;

	fn truncate(&mut self, truncate: bool) -> &mut Self;

	fn write(&mut self, write: bool) -> &mut Self;
}
