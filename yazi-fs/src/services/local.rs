use std::{io, path::{Path, PathBuf}};

pub struct Local;

impl Local {
	#[inline]
	pub async fn canonicalize(path: impl AsRef<Path>) -> io::Result<PathBuf> {
		tokio::fs::canonicalize(path).await
	}

	#[inline]
	pub async fn create(path: impl AsRef<Path>) -> io::Result<tokio::fs::File> {
		tokio::fs::File::create(path).await
	}

	#[inline]
	pub async fn create_dir(path: impl AsRef<Path>) -> io::Result<()> {
		tokio::fs::create_dir(path).await
	}

	#[inline]
	pub async fn create_dir_all(path: impl AsRef<Path>) -> io::Result<()> {
		tokio::fs::create_dir_all(path).await
	}

	#[inline]
	pub async fn hard_link(original: impl AsRef<Path>, link: impl AsRef<Path>) -> io::Result<()> {
		tokio::fs::hard_link(original, link).await
	}

	#[inline]
	pub async fn metadata(url: impl AsRef<Path>) -> io::Result<std::fs::Metadata> {
		tokio::fs::metadata(url).await
	}

	#[inline]
	pub async fn open(path: impl AsRef<Path>) -> io::Result<tokio::fs::File> {
		tokio::fs::File::open(path).await
	}

	#[inline]
	pub async fn read(path: impl AsRef<Path>) -> io::Result<Vec<u8>> { tokio::fs::read(path).await }

	#[inline]
	pub async fn read_dir(path: impl AsRef<Path>) -> io::Result<tokio::fs::ReadDir> {
		tokio::fs::read_dir(path).await
	}

	#[inline]
	pub fn read_dir_sync(path: impl AsRef<Path>) -> io::Result<std::fs::ReadDir> {
		std::fs::read_dir(path)
	}

	#[inline]
	pub async fn read_link(url: impl AsRef<Path>) -> io::Result<PathBuf> {
		tokio::fs::read_link(url).await
	}

	#[inline]
	pub async fn read_to_string(path: impl AsRef<Path>) -> io::Result<String> {
		tokio::fs::read_to_string(path).await
	}

	#[inline]
	pub async fn remove_dir(path: impl AsRef<Path>) -> io::Result<()> {
		tokio::fs::remove_dir(path).await
	}

	#[inline]
	pub async fn remove_dir_all(path: impl AsRef<Path>) -> io::Result<()> {
		tokio::fs::remove_dir_all(path).await
	}

	#[inline]
	pub async fn remove_file(path: impl AsRef<Path>) -> io::Result<()> {
		tokio::fs::remove_file(path).await
	}

	#[inline]
	pub async fn rename(from: impl AsRef<Path>, to: impl AsRef<Path>) -> io::Result<()> {
		tokio::fs::rename(from, to).await
	}

	#[inline]
	pub async fn symlink_dir(original: impl AsRef<Path>, link: impl AsRef<Path>) -> io::Result<()> {
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
	pub async fn symlink_file(original: impl AsRef<Path>, link: impl AsRef<Path>) -> io::Result<()> {
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
	pub async fn symlink_metadata(path: impl AsRef<Path>) -> io::Result<std::fs::Metadata> {
		tokio::fs::symlink_metadata(path).await
	}

	#[inline]
	pub fn symlink_metadata_sync(path: impl AsRef<Path>) -> io::Result<std::fs::Metadata> {
		std::fs::symlink_metadata(path)
	}

	#[inline]
	pub async fn write(path: impl AsRef<Path>, contents: impl AsRef<[u8]>) -> io::Result<()> {
		tokio::fs::write(path, contents).await
	}
}
