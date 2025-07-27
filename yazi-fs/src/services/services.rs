use std::io;

use yazi_shared::url::Url;

use crate::services::Local;

#[inline]
pub async fn canonicalize(url: impl AsRef<Url>) -> io::Result<Url> {
	Local::canonicalize(url.as_ref()).await.map(Into::into)
}

#[inline]
pub async fn create(url: impl AsRef<Url>) -> io::Result<tokio::fs::File> {
	Local::create(url.as_ref()).await
}

#[inline]
pub async fn create_dir(url: impl AsRef<Url>) -> io::Result<()> {
	Local::create_dir(url.as_ref()).await
}

#[inline]
pub async fn create_dir_all(url: impl AsRef<Url>) -> io::Result<()> {
	Local::create_dir_all(url.as_ref()).await
}

#[inline]
pub async fn hard_link(original: impl AsRef<Url>, link: impl AsRef<Url>) -> io::Result<()> {
	Local::hard_link(original.as_ref(), link.as_ref()).await
}

#[inline]
pub async fn metadata(url: impl AsRef<Url>) -> io::Result<std::fs::Metadata> {
	Local::metadata(url.as_ref()).await
}

#[inline]
pub async fn open(url: impl AsRef<Url>) -> io::Result<tokio::fs::File> {
	Local::open(url.as_ref()).await
}

#[inline]
pub async fn read_dir(url: impl AsRef<Url>) -> io::Result<tokio::fs::ReadDir> {
	Local::read_dir(url.as_ref()).await
}

#[inline]
pub fn read_dir_sync(url: impl AsRef<Url>) -> io::Result<std::fs::ReadDir> {
	Local::read_dir_sync(url.as_ref())
}

#[inline]
pub async fn read_link(url: impl AsRef<Url>) -> io::Result<Url> {
	Local::read_link(url.as_ref()).await.map(Into::into)
}

#[inline]
pub async fn remove_dir(url: impl AsRef<Url>) -> io::Result<()> {
	Local::remove_dir(url.as_ref()).await
}

#[inline]
pub async fn remove_dir_all(url: impl AsRef<Url>) -> io::Result<()> {
	Local::remove_dir_all(url.as_ref()).await
}

#[inline]
pub async fn remove_file(url: impl AsRef<Url>) -> io::Result<()> {
	Local::remove_file(url.as_ref()).await
}

#[inline]
pub async fn rename(from: impl AsRef<Url>, to: impl AsRef<Url>) -> io::Result<()> {
	Local::rename(from.as_ref(), to.as_ref()).await
}

#[inline]
pub async fn symlink_dir(original: impl AsRef<Url>, link: impl AsRef<Url>) -> io::Result<()> {
	Local::symlink_dir(original.as_ref(), link.as_ref()).await
}

#[inline]
pub async fn symlink_file(original: impl AsRef<Url>, link: impl AsRef<Url>) -> io::Result<()> {
	Local::symlink_file(original.as_ref(), link.as_ref()).await
}

#[inline]
pub async fn symlink_metadata(url: impl AsRef<Url>) -> io::Result<std::fs::Metadata> {
	Local::symlink_metadata(url.as_ref()).await
}

pub fn symlink_metadata_sync(url: impl AsRef<Url>) -> io::Result<std::fs::Metadata> {
	Local::symlink_metadata_sync(url.as_ref())
}

#[inline]
pub async fn write(url: impl AsRef<Url>, contents: impl AsRef<[u8]>) -> io::Result<()> {
	Local::write(url.as_ref(), contents).await
}
