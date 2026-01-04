use std::io;

use tokio::{io::{AsyncRead, AsyncSeek, AsyncWrite, AsyncWriteExt}, sync::mpsc};
use yazi_macro::ok_or_not_found;
use yazi_shared::{path::{AsPath, PathBufDyn}, strand::{AsStrand, StrandCow}, url::{AsUrl, Url, UrlBuf}};

use crate::{cha::{Cha, ChaType}, provider::{Attrs, Capabilities}};

pub trait Provider: Sized {
	type File: AsyncRead + AsyncSeek + AsyncWrite + Unpin;
	type Gate: FileBuilder<File = Self::File>;
	type ReadDir: DirReader + 'static;
	type UrlCow;
	type Me<'a>: Provider;

	fn absolute(&self) -> impl Future<Output = io::Result<Self::UrlCow>>;

	fn canonicalize(&self) -> impl Future<Output = io::Result<UrlBuf>>;

	fn capabilities(&self) -> Capabilities;

	fn casefold(&self) -> impl Future<Output = io::Result<UrlBuf>>;

	fn copy<P>(&self, to: P, attrs: Attrs) -> impl Future<Output = io::Result<u64>>
	where
		P: AsPath;

	fn copy_with_progress<P, A>(
		&self,
		to: P,
		attrs: A,
	) -> io::Result<mpsc::Receiver<io::Result<u64>>>
	where
		P: AsPath,
		A: Into<Attrs>;

	fn create(&self) -> impl Future<Output = io::Result<Self::File>> {
		async move { self.gate().write(true).create(true).truncate(true).open(self.url()).await }
	}

	fn create_dir(&self) -> impl Future<Output = io::Result<()>>;

	fn create_dir_all(&self) -> impl Future<Output = io::Result<()>> {
		async move {
			let mut url = self.url();
			if url.loc().is_empty() {
				return Ok(());
			}

			let mut stack = Vec::new();
			loop {
				match Self::new(url).await?.create_dir().await {
					Ok(()) => break,
					Err(e) if e.kind() == io::ErrorKind::NotFound => {
						if let Some(parent) = url.parent() {
							stack.push(url);
							url = parent;
						} else {
							return Err(io::Error::other("failed to create whole tree"));
						}
					}
					Err(_) if Self::new(url).await?.metadata().await.is_ok_and(|m| m.is_dir()) => break,
					Err(e) => return Err(e),
				}
			}

			while let Some(u) = stack.pop() {
				match Self::new(u).await?.create_dir().await {
					Ok(()) => {}
					Err(_) if Self::new(u).await?.metadata().await.is_ok_and(|m| m.is_dir()) => {}
					Err(e) => return Err(e),
				}
			}

			Ok(())
		}
	}

	fn create_new(&self) -> impl Future<Output = io::Result<Self::File>> {
		async move { self.gate().write(true).create_new(true).open(self.url()).await }
	}

	fn gate(&self) -> Self::Gate { Self::Gate::default() }

	fn hard_link<P>(&self, to: P) -> impl Future<Output = io::Result<()>>
	where
		P: AsPath;

	fn metadata(&self) -> impl Future<Output = io::Result<Cha>>;

	fn new<'a>(url: Url<'a>) -> impl Future<Output = io::Result<Self::Me<'a>>>;

	fn open(&self) -> impl Future<Output = io::Result<Self::File>> {
		async move { self.gate().read(true).open(self.url()).await }
	}

	fn read_dir(self) -> impl Future<Output = io::Result<Self::ReadDir>>;

	fn read_link(&self) -> impl Future<Output = io::Result<PathBufDyn>>;

	fn remove_dir(&self) -> impl Future<Output = io::Result<()>>;

	fn remove_dir_all(&self) -> impl Future<Output = io::Result<()>> {
		async fn remove_dir_all_impl<P>(url: Url<'_>) -> io::Result<()>
		where
			P: Provider,
		{
			let mut it = ok_or_not_found!(P::new(url).await?.read_dir().await, return Ok(()));
			while let Some(child) = it.next().await? {
				let ft = ok_or_not_found!(child.file_type().await, continue);
				let result = if ft.is_dir() {
					Box::pin(remove_dir_all_impl::<P>(child.url().as_url())).await
				} else {
					P::new(child.url().as_url()).await?.remove_file().await
				};

				() = ok_or_not_found!(result);
			}

			Ok(ok_or_not_found!(P::new(url).await?.remove_dir().await))
		}

		async move {
			let cha = ok_or_not_found!(self.symlink_metadata().await, return Ok(()));
			if cha.is_link() {
				self.remove_file().await
			} else {
				remove_dir_all_impl::<Self>(self.url()).await
			}
		}
	}

	fn remove_dir_clean(&self) -> impl Future<Output = io::Result<()>> {
		let root = self.url().to_owned();

		async move {
			let mut stack = vec![(root, false)];
			let mut result = Ok(());

			while let Some((dir, visited)) = stack.pop() {
				let Ok(provider) = Self::new(dir.as_url()).await else {
					continue;
				};

				if visited {
					result = provider.remove_dir().await;
				} else if let Ok(mut it) = provider.read_dir().await {
					stack.push((dir, true));
					while let Ok(Some(ent)) = it.next().await {
						if ent.file_type().await.is_ok_and(|t| t.is_dir()) {
							stack.push((ent.url(), false));
						}
					}
				}
			}
			result
		}
	}

	fn remove_file(&self) -> impl Future<Output = io::Result<()>>;

	fn rename<P>(&self, to: P) -> impl Future<Output = io::Result<()>>
	where
		P: AsPath;

	fn symlink<S, F>(&self, original: S, _is_dir: F) -> impl Future<Output = io::Result<()>>
	where
		S: AsStrand,
		F: AsyncFnOnce() -> io::Result<bool>;

	fn symlink_dir<S>(&self, original: S) -> impl Future<Output = io::Result<()>>
	where
		S: AsStrand,
	{
		self.symlink(original, async || Ok(true))
	}

	fn symlink_file<S>(&self, original: S) -> impl Future<Output = io::Result<()>>
	where
		S: AsStrand,
	{
		self.symlink(original, async || Ok(false))
	}

	fn symlink_metadata(&self) -> impl Future<Output = io::Result<Cha>>;

	fn trash(&self) -> impl Future<Output = io::Result<()>>;

	fn url(&self) -> Url<'_>;

	fn write<C>(&self, contents: C) -> impl Future<Output = io::Result<()>>
	where
		C: AsRef<[u8]>,
	{
		async move { self.create().await?.write_all(contents.as_ref()).await }
	}
}

// --- DirReader
pub trait DirReader {
	type Entry: FileHolder;

	#[must_use]
	fn next(&mut self) -> impl Future<Output = io::Result<Option<Self::Entry>>>;
}

// --- FileHolder
pub trait FileHolder {
	#[must_use]
	fn file_type(&self) -> impl Future<Output = io::Result<ChaType>>;

	#[must_use]
	fn metadata(&self) -> impl Future<Output = io::Result<Cha>>;

	#[must_use]
	fn name(&self) -> StrandCow<'_>;

	#[must_use]
	fn path(&self) -> PathBufDyn;

	#[must_use]
	fn url(&self) -> UrlBuf;
}

// --- FileBuilder
pub trait FileBuilder: Sized + Default {
	type File: AsyncRead + AsyncSeek + AsyncWrite + Unpin;

	fn append(&mut self, append: bool) -> &mut Self;

	fn attrs(&mut self, attrs: Attrs) -> &mut Self;

	fn create(&mut self, create: bool) -> &mut Self;

	fn create_new(&mut self, create_new: bool) -> &mut Self;

	fn open<U>(&self, url: U) -> impl Future<Output = io::Result<Self::File>>
	where
		U: AsUrl;

	fn read(&mut self, read: bool) -> &mut Self;

	fn truncate(&mut self, truncate: bool) -> &mut Self;

	fn write(&mut self, write: bool) -> &mut Self;
}
