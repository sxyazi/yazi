use std::{io, path::{Path, PathBuf}};

use crate::{cha::Cha, provider::local::{Gate, ReadDir, ReadDirSync, RwFile}};

pub struct Local;

impl Local {
	#[inline]
	pub fn cache<P>(_: P) -> Option<PathBuf>
	where
		P: AsRef<Path>,
	{
		None
	}

	#[inline]
	pub async fn canonicalize<P>(path: P) -> io::Result<PathBuf>
	where
		P: AsRef<Path>,
	{
		tokio::fs::canonicalize(path).await
	}

	#[inline]
	pub async fn copy<P, Q>(from: P, to: Q, cha: Cha) -> io::Result<u64>
	where
		P: AsRef<Path>,
		Q: AsRef<Path>,
	{
		let from = from.as_ref().to_owned();
		let to = to.as_ref().to_owned();
		Self::copy_impl(from, to, cha).await
	}

	async fn copy_impl(from: PathBuf, to: PathBuf, cha: Cha) -> io::Result<u64> {
		let mut ft = std::fs::FileTimes::new();
		cha.atime.map(|t| ft = ft.set_accessed(t));
		cha.mtime.map(|t| ft = ft.set_modified(t));
		#[cfg(target_os = "macos")]
		{
			use std::os::macos::fs::FileTimesExt;
			cha.btime.map(|t| ft = ft.set_created(t));
		}
		#[cfg(windows)]
		{
			use std::os::windows::fs::FileTimesExt;
			cha.btime.map(|t| ft = ft.set_created(t));
		}

		#[cfg(any(target_os = "linux", target_os = "android"))]
		{
			use std::os::{fd::AsRawFd, unix::fs::OpenOptionsExt};

			tokio::task::spawn_blocking(move || {
				let mut reader = std::fs::File::open(from)?;
				let mut writer = std::fs::OpenOptions::new()
				.mode(cha.mode as u32)  // Do not remove `as u32`, https://github.com/termux/termux-packages/pull/22481
				.write(true)
				.create(true)
				.truncate(true)
				.open(to)?;

				let written = std::io::copy(&mut reader, &mut writer)?;
				unsafe { libc::fchmod(writer.as_raw_fd(), cha.mode) };
				writer.set_times(ft).ok();

				Ok(written)
			})
			.await?
		}

		#[cfg(not(any(target_os = "linux", target_os = "android")))]
		{
			tokio::task::spawn_blocking(move || {
				let written = std::fs::copy(from, &to)?;
				std::fs::File::options().write(true).open(to).and_then(|f| f.set_times(ft)).ok();
				Ok(written)
			})
			.await?
		}
	}

	#[inline]
	pub async fn create<P>(path: P) -> io::Result<RwFile>
	where
		P: AsRef<Path>,
	{
		Gate::default().write(true).create(true).truncate(true).open(path).await.map(Into::into)
	}

	#[inline]
	pub async fn create_dir<P>(path: P) -> io::Result<()>
	where
		P: AsRef<Path>,
	{
		tokio::fs::create_dir(path).await
	}

	#[inline]
	pub async fn create_dir_all<P>(path: P) -> io::Result<()>
	where
		P: AsRef<Path>,
	{
		tokio::fs::create_dir_all(path).await
	}

	#[inline]
	pub async fn hard_link<P, Q>(original: P, link: Q) -> io::Result<()>
	where
		P: AsRef<Path>,
		Q: AsRef<Path>,
	{
		tokio::fs::hard_link(original, link).await
	}

	#[inline]
	pub async fn metadata<P>(url: P) -> io::Result<std::fs::Metadata>
	where
		P: AsRef<Path>,
	{
		tokio::fs::metadata(url).await
	}

	#[inline]
	pub async fn open<P>(path: P) -> io::Result<RwFile>
	where
		P: AsRef<Path>,
	{
		Gate::default().read(true).open(path).await.map(Into::into)
	}

	#[inline]
	pub async fn read<P>(path: P) -> io::Result<Vec<u8>>
	where
		P: AsRef<Path>,
	{
		tokio::fs::read(path).await
	}

	#[inline]
	pub async fn read_dir<P>(path: P) -> io::Result<ReadDir>
	where
		P: AsRef<Path>,
	{
		tokio::fs::read_dir(path).await.map(Into::into)
	}

	#[inline]
	pub fn read_dir_sync<P>(path: P) -> io::Result<ReadDirSync>
	where
		P: AsRef<Path>,
	{
		std::fs::read_dir(path).map(Into::into)
	}

	#[inline]
	pub async fn read_link<P>(url: P) -> io::Result<PathBuf>
	where
		P: AsRef<Path>,
	{
		tokio::fs::read_link(url).await
	}

	#[inline]
	pub async fn read_to_string<P>(path: P) -> io::Result<String>
	where
		P: AsRef<Path>,
	{
		tokio::fs::read_to_string(path).await
	}

	#[inline]
	pub async fn remove_dir<P>(path: P) -> io::Result<()>
	where
		P: AsRef<Path>,
	{
		tokio::fs::remove_dir(path).await
	}

	#[inline]
	pub async fn remove_dir_all<P>(path: P) -> io::Result<()>
	where
		P: AsRef<Path>,
	{
		tokio::fs::remove_dir_all(path).await
	}

	#[inline]
	pub async fn remove_file<P>(path: P) -> io::Result<()>
	where
		P: AsRef<Path>,
	{
		tokio::fs::remove_file(path).await
	}

	#[inline]
	pub async fn rename<P, Q>(from: P, to: Q) -> io::Result<()>
	where
		P: AsRef<Path>,
		Q: AsRef<Path>,
	{
		tokio::fs::rename(from, to).await
	}

	#[inline]
	pub async fn same<P, Q>(a: P, b: Q) -> io::Result<bool>
	where
		P: AsRef<Path>,
		Q: AsRef<Path>,
	{
		Self::same_impl(a.as_ref(), b.as_ref()).await
	}

	#[cfg(unix)]
	async fn same_impl(a: &Path, b: &Path) -> io::Result<bool> {
		use std::os::unix::fs::MetadataExt;

		let (a_, b_) = (tokio::fs::symlink_metadata(a).await?, tokio::fs::symlink_metadata(b).await?);
		Ok(
			a_.ino() == b_.ino()
				&& a_.dev() == b_.dev()
				&& tokio::fs::canonicalize(a).await? == tokio::fs::canonicalize(b).await?,
		)
	}

	#[cfg(windows)]
	async fn same_impl(a: &Path, b: &Path) -> io::Result<bool> {
		use std::{ffi::OsString, os::windows::{ffi::OsStringExt, fs::OpenOptionsExt, io::AsRawHandle}};

		use windows_sys::Win32::{Foundation::{HANDLE, MAX_PATH}, Storage::FileSystem::{FILE_FLAG_BACKUP_SEMANTICS, FILE_FLAG_OPEN_REPARSE_POINT, GetFinalPathNameByHandleW, VOLUME_NAME_DOS}};

		async fn final_name(path: &Path) -> io::Result<PathBuf> {
			let path = path.to_owned();
			tokio::task::spawn_blocking(move || {
				let file = std::fs::OpenOptions::new()
					.access_mode(0)
					.custom_flags(FILE_FLAG_BACKUP_SEMANTICS | FILE_FLAG_OPEN_REPARSE_POINT)
					.open(path)?;

				let mut buf = [0u16; MAX_PATH as usize];
				let len = unsafe {
					GetFinalPathNameByHandleW(
						file.as_raw_handle() as HANDLE,
						buf.as_mut_ptr(),
						buf.len() as u32,
						VOLUME_NAME_DOS,
					)
				};

				if len == 0 {
					Err(io::Error::last_os_error())
				} else {
					Ok(PathBuf::from(OsString::from_wide(&buf[0..len as usize])))
				}
			})
			.await?
		}

		Ok(final_name(a).await? == final_name(b).await?)
	}

	#[inline]
	pub async fn symlink_dir<P, Q>(original: P, link: Q) -> io::Result<()>
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
	pub async fn symlink_file<P, Q>(original: P, link: Q) -> io::Result<()>
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
	pub async fn symlink_metadata<P>(path: P) -> io::Result<std::fs::Metadata>
	where
		P: AsRef<Path>,
	{
		tokio::fs::symlink_metadata(path).await
	}

	#[inline]
	pub fn symlink_metadata_sync<P>(path: P) -> io::Result<std::fs::Metadata>
	where
		P: AsRef<Path>,
	{
		std::fs::symlink_metadata(path)
	}

	pub async fn trash<P>(path: P) -> io::Result<()>
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
	pub async fn write<P, C>(path: P, contents: C) -> io::Result<()>
	where
		P: AsRef<Path>,
		C: AsRef<[u8]>,
	{
		tokio::fs::write(path, contents).await
	}
}
