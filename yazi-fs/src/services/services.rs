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
pub async fn metadata(url: impl AsRef<Url>) -> io::Result<std::fs::Metadata> {
	Local::metadata(url.as_ref()).await
}

#[inline]
pub async fn open(url: impl AsRef<Url>) -> io::Result<tokio::fs::File> {
	tokio::fs::File::open(url.as_ref()).await
}

#[inline]
pub async fn read_dir(url: impl AsRef<Url>) -> io::Result<tokio::fs::ReadDir> {
	Local::read_dir(url.as_ref()).await
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
pub async fn symlink_metadata(url: impl AsRef<Url>) -> io::Result<std::fs::Metadata> {
	Local::symlink_metadata(url.as_ref()).await
}

#[inline]
pub async fn write(url: impl AsRef<Url>, contents: impl AsRef<[u8]>) -> io::Result<()> {
	Local::write(url.as_ref(), contents).await
}
