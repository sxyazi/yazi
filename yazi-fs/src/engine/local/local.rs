use std::{fs::FileTimes, io, path::Path, sync::Arc};

use tokio::sync::mpsc;
use yazi_shared::{auth::AuthKind, path::{DynPath, PathBufDyn}, strand::AsStrand, url::{Url, UrlBuf, UrlCow}};

use crate::{cha::{Cha, ChaMode}, engine::{Attrs, Capabilities, Engine}};

#[derive(Clone)]
pub struct Local<'a> {
	url:  Url<'a>,
	path: &'a Path,
}

impl<'a> Engine for Local<'a> {
	type Demand = super::Demand;
	type File = tokio::fs::File;
	type Me<'b> = Local<'b>;
	type ReadDir = super::ReadDir;
	type UrlCow = UrlCow<'a>;

	async fn absolute(&self) -> io::Result<Self::UrlCow> {
		super::try_absolute(self.url)
			.ok_or_else(|| io::Error::other("Cannot get absolute path for local URL"))
	}

	#[inline]
	async fn canonicalize(&self) -> io::Result<UrlBuf> {
		tokio::fs::canonicalize(self.path).await.map(Into::into)
	}

	#[inline]
	async fn capabilities(&self) -> io::Result<Capabilities> {
		Ok(Capabilities {
			symlink:          true,
			hard_link:        true,
			trash:            true,
			copy_progressive: true,
		})
	}

	async fn casefold(&self) -> io::Result<UrlBuf> {
		super::casefold(self.path).await.map(Into::into)
	}

	#[inline]
	async fn copy<P>(&self, to: P, attrs: Attrs) -> io::Result<u64>
	where
		P: DynPath,
	{
		let to = to.dyn_path().to_os_owned()?;
		let from = self.path.to_owned();
		super::copy_impl(from, to, attrs).await
	}

	fn copy_progressive<P, A>(&self, to: P, attrs: A) -> io::Result<mpsc::Receiver<io::Result<u64>>>
	where
		P: DynPath,
		A: Into<Attrs>,
	{
		let to = to.dyn_path().to_os_owned()?;
		let from = self.path.to_owned();
		Ok(super::copy_progressive_impl(from, to, attrs.into()))
	}

	#[inline]
	async fn create_dir(&self) -> io::Result<()> { tokio::fs::create_dir(self.path).await }

	#[inline]
	async fn create_dir_all(&self) -> io::Result<()> { tokio::fs::create_dir_all(self.path).await }

	#[inline]
	async fn hard_link<P>(&self, to: P) -> io::Result<()>
	where
		P: DynPath,
	{
		let to = to.dyn_path().as_os()?;

		tokio::fs::hard_link(self.path, to).await
	}

	#[inline]
	async fn metadata(&self) -> io::Result<Cha> {
		Ok(Cha::new(self.path.file_name().unwrap_or_default(), tokio::fs::metadata(self.path).await?))
	}

	#[inline]
	async fn new<'b>(url: Url<'b>) -> io::Result<Self::Me<'b>> {
		match url {
			Url::Regular(loc) | Url::Search { loc, .. } => Ok(Self::Me { url, path: loc.as_inner() }),
			Url::Mount { .. } | Url::Hub { .. } | Url::Scope { .. } | Url::Sftp { .. } => {
				Err(io::Error::new(io::ErrorKind::InvalidInput, format!("Not a local URL: {url:?}")))
			}
		}
	}

	#[inline]
	async fn read_dir(self) -> io::Result<Self::ReadDir> {
		Ok(match self.url.kind() {
			AuthKind::Regular => Self::ReadDir::Regular(tokio::fs::read_dir(self.path).await?),
			AuthKind::Search => Self::ReadDir::Others {
				reader: tokio::fs::read_dir(self.path).await?,
				dir:    Arc::new(self.url.to_owned()),
			},
			AuthKind::Mount | AuthKind::Hub | AuthKind::Scope | AuthKind::Sftp => Err(io::Error::new(
				io::ErrorKind::InvalidInput,
				format!("Not a local URL: {:?}", self.url),
			))?,
		})
	}

	#[inline]
	async fn read_link(&self) -> io::Result<PathBufDyn> {
		Ok(tokio::fs::read_link(self.path).await?.into())
	}

	#[inline]
	async fn remove_dir(&self) -> io::Result<()> { tokio::fs::remove_dir(self.path).await }

	#[inline]
	async fn remove_dir_all(&self) -> io::Result<()> { tokio::fs::remove_dir_all(self.path).await }

	#[inline]
	async fn remove_file(&self) -> io::Result<()> { tokio::fs::remove_file(self.path).await }

	#[inline]
	async fn rename<P>(&self, to: P) -> io::Result<()>
	where
		P: DynPath,
	{
		let to = to.dyn_path().as_os()?;

		tokio::fs::rename(self.path, to).await
	}

	async fn set_attrs(&self, attrs: Attrs) -> io::Result<()> {
		let (mode, times) = (attrs.mode, attrs.try_into());
		if mode.is_none() && times.is_err() {
			return Ok(());
		}

		let path = self.path.to_owned();
		tokio::task::spawn_blocking(move || {
			let a = mode.map_or(Ok(()), |mode| Self::set_mode(&path, mode));
			let b = times.map_or(Ok(()), |times| Self::set_times(&path, times));
			a.and(b)
		})
		.await?
	}

	#[inline]
	async fn symlink<S, F>(&self, original: S, _is_dir: F) -> io::Result<()>
	where
		S: AsStrand,
		F: AsyncFnOnce() -> io::Result<bool>,
	{
		#[cfg(unix)]
		{
			let original = original.as_strand().as_os()?;
			tokio::fs::symlink(original, self.path).await
		}
		#[cfg(windows)]
		if _is_dir().await? {
			self.symlink_dir(original).await
		} else {
			self.symlink_file(original).await
		}
	}

	#[inline]
	async fn symlink_dir<S>(&self, original: S) -> io::Result<()>
	where
		S: AsStrand,
	{
		let original = original.as_strand().as_os()?;

		#[cfg(unix)]
		{
			tokio::fs::symlink(original, self.path).await
		}
		#[cfg(windows)]
		{
			tokio::fs::symlink_dir(original, self.path).await
		}
	}

	#[inline]
	async fn symlink_file<S>(&self, original: S) -> io::Result<()>
	where
		S: AsStrand,
	{
		let original = original.as_strand().as_os()?;

		#[cfg(unix)]
		{
			tokio::fs::symlink(original, self.path).await
		}
		#[cfg(windows)]
		{
			tokio::fs::symlink_file(original, self.path).await
		}
	}

	#[inline]
	async fn symlink_metadata(&self) -> io::Result<Cha> {
		Ok(Cha::new(
			self.path.file_name().unwrap_or_default(),
			tokio::fs::symlink_metadata(self.path).await?,
		))
	}

	async fn trash(&self) -> io::Result<()> {
		let path = self.path.to_owned();
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
	fn url(&self) -> Url<'_> { self.url }

	#[inline]
	async fn write<C>(&self, contents: C) -> io::Result<()>
	where
		C: AsRef<[u8]>,
	{
		tokio::fs::write(self.path, contents).await
	}
}

impl<'a> Local<'a> {
	#[inline]
	pub async fn read(&self) -> io::Result<Vec<u8>> { tokio::fs::read(self.path).await }

	#[inline]
	pub async fn read_to_string(&self) -> io::Result<String> {
		tokio::fs::read_to_string(self.path).await
	}

	#[inline]
	pub fn regular<P>(path: &'a P) -> Self
	where
		P: ?Sized + AsRef<Path>,
	{
		Self { url: Url::regular(path), path: path.as_ref() }
	}

	fn set_mode(path: &Path, mode: ChaMode) -> io::Result<()> {
		#[cfg(unix)]
		{
			std::fs::set_permissions(path, mode.into())
		}

		#[cfg(windows)]
		{
			use std::os::windows::ffi::OsStrExt;

			let path: Vec<u16> = path.as_os_str().encode_wide().chain(Some(0)).collect();
			let perm = if mode.contains(ChaMode::U_WRITE) { libc::S_IWRITE } else { libc::S_IREAD };

			let result = unsafe { libc::wchmod(path.as_ptr(), perm) };
			if result == 0 { Ok(()) } else { Err(io::Error::last_os_error()) }
		}
	}

	fn set_times(path: &Path, times: FileTimes) -> io::Result<()> {
		std::fs::File::open(path)?.set_times(times)
	}
}
