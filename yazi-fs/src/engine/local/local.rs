use std::{io, path::Path};

use yazi_shared::{auth::AuthKind, path::{DynPath, PathBufDyn, PathDynError}, strand::AsStrand, url::{AsUrl, Url, UrlBuf, UrlCow, UrlLike}};

use crate::{casefold::Casefold, copier::Copier, engine::{Attrs, Capabilities, Engine, RwFile, Transit}, identity::Identity, remover::Removal, stat::{Stat, StatMode}, symlinker::Symlinker};

#[derive(Clone)]
pub struct Local<'a> {
	url:  Url<'a>,
	path: &'a Path,
}

impl AsUrl for Local<'_> {
	fn as_url(&self) -> Url<'_> { self.url }
}

impl UrlLike for Local<'_> {}

impl<'a> Engine for Local<'a> {
	type Demand = super::Demand;
	type File = RwFile;
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
		Ok(Capabilities::for_kind(AuthKind::Regular))
	}

	async fn casefold(&self) -> io::Result<UrlBuf> {
		Casefold::casefold(self.path).await.map(Into::into)
	}

	async fn copy_to(&self, to: Url<'_>, attrs: Attrs) -> io::Result<Transit> {
		let Some(to) = to.as_local() else { return Ok(Transit::unsupported()) };

		Ok(Copier::new(self.path, to, attrs)?.spawn())
	}

	async fn copy_from(&self, from: Url<'_>, attrs: Attrs) -> io::Result<Transit> {
		let Some(from) = from.as_local() else { return Ok(Transit::unsupported()) };

		Ok(Copier::new(from, self.path, attrs)?.spawn())
	}

	#[inline]
	async fn create_dir(&self) -> io::Result<()> { tokio::fs::create_dir(self.path).await }

	#[inline]
	async fn create_dir_all(&self) -> io::Result<()> { tokio::fs::create_dir_all(self.path).await }

	#[inline]
	async fn create_file(&self) -> io::Result<Self::File> { RwFile::create(self.path).await }

	#[inline]
	async fn create_file_new(&self) -> io::Result<Self::File> { RwFile::create_new(self.path).await }

	#[inline]
	async fn hardlink<P>(&self, to: P, follow: bool, force: bool) -> io::Result<()>
	where
		P: DynPath,
	{
		let from = self.path.to_owned();
		let to = to.dyn_path().to_os_owned()?;

		tokio::task::spawn_blocking(move || Self::hardlink(&from, &to, follow, force)).await?
	}

	#[inline]
	async fn lstat(&self) -> io::Result<Stat> {
		Ok(Stat::new(
			self.path.file_name().unwrap_or_default(),
			tokio::fs::symlink_metadata(self.path).await?,
		))
	}

	#[inline]
	async fn new<'b>(url: Url<'b>) -> io::Result<Self::Me<'b>> {
		let path = url.as_local().ok_or_else(|| {
			io::Error::new(io::ErrorKind::InvalidInput, format!("Not a local URL: {url}"))
		})?;

		Ok(Self::Me { url, path })
	}

	#[inline]
	async fn read_dir(self) -> io::Result<Self::ReadDir> {
		Ok(super::ReadDir(tokio::fs::read_dir(self.path).await?))
	}

	#[inline]
	async fn read_link(&self) -> io::Result<PathBufDyn> {
		Ok(tokio::fs::read_link(self.path).await?.into())
	}

	#[inline]
	async fn remove_dir(&self) -> io::Result<()> { tokio::fs::remove_dir(self.path).await }

	#[inline]
	async fn remove_dir_all(&self) -> io::Result<Removal> {
		Ok(crate::remover::Remover::spawn(self.path))
	}

	#[inline]
	async fn remove_dir_clean(&self) -> io::Result<()> {
		let path = self.path.to_owned();
		tokio::task::spawn_blocking(move || super::remove_dir_clean_impl(&path)).await?
	}

	#[inline]
	async fn remove_file(&self) -> io::Result<()> { tokio::fs::remove_file(self.path).await }

	async fn rename<U>(&self, to: U) -> io::Result<UrlBuf>
	where
		U: AsUrl,
	{
		let from = self.path.to_owned();
		let UrlBuf::Os { loc, auth } = to.as_url().to_owned() else {
			return Err(PathDynError::AsOs.into());
		};

		tokio::task::spawn_blocking(move || {
			std::fs::rename(&from, &*loc)?;
			Ok(UrlBuf::Os { loc, auth })
		})
		.await?
	}

	async fn same_entry(&self, other: Url<'_>) -> io::Result<bool> {
		match other.as_local() {
			Some(path) => Identity::same_entry(self.path, path).await,
			None => Ok(false),
		}
	}

	async fn same_file(&self, other: Url<'_>) -> io::Result<bool> {
		match other.as_local() {
			Some(path) => Identity::same_file(self.path, path).await,
			None => Ok(false),
		}
	}

	async fn set_attrs(&self, attrs: Attrs) -> io::Result<()> {
		let (mode, times) = (attrs.mode, attrs.try_into());
		if mode.is_none() && times.is_err() {
			return Ok(());
		}

		let path = self.path.to_owned();
		tokio::task::spawn_blocking(move || {
			let a = mode.map_or(Ok(()), |mode| Self::set_mode(&path, mode));
			let b = times.map_or(Ok(()), |times| std::fs::set_times(&path, times));
			a.and(b)
		})
		.await?
	}

	#[inline]
	async fn stat(&self) -> io::Result<Stat> {
		Ok(Stat::new(self.path.file_name().unwrap_or_default(), tokio::fs::metadata(self.path).await?))
	}

	#[inline]
	async fn symlink_dir<S>(&self, original: S, force: bool) -> io::Result<()>
	where
		S: AsStrand,
	{
		let original = original.as_strand().to_os()?;
		let link = self.path.to_owned();

		tokio::task::spawn_blocking(move || Symlinker::dir(&original, &link, force)).await?
	}

	#[inline]
	async fn symlink_file<S>(&self, original: S, force: bool) -> io::Result<()>
	where
		S: AsStrand,
	{
		let original = original.as_strand().to_os()?;
		let link = self.path.to_owned();

		tokio::task::spawn_blocking(move || Symlinker::file(&original, &link, force)).await?
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

	fn set_mode(path: &Path, mode: StatMode) -> io::Result<()> {
		#[cfg(unix)]
		{
			std::fs::set_permissions(path, mode.into())
		}

		#[cfg(windows)]
		{
			use yazi_shim::ToWide;

			let path = path.to_wide();
			let perm = if mode.contains(StatMode::U_WRITE) { libc::S_IWRITE } else { libc::S_IREAD };

			let result = unsafe { libc::wchmod(path.as_ptr(), perm) };
			if result == 0 { Ok(()) } else { Err(io::Error::last_os_error()) }
		}
	}
}
