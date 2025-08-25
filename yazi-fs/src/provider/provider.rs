use std::{io, path::{Path, PathBuf}};

use yazi_shared::url::{Url, UrlBuf};

use crate::{cha::Cha, provider::{ReadDir, ReadDirSync, RwFile, local::{self, Local}}};

#[inline]
pub fn cache<'a, U>(url: U) -> Option<PathBuf>
where
	U: Into<Url<'a>>,
{
	if let Some(path) = url.into().as_path() { Local::cache(path) } else { None }
}

#[inline]
pub async fn canonicalize<'a, U>(url: U) -> io::Result<UrlBuf>
where
	U: Into<Url<'a>>,
{
	if let Some(path) = url.into().as_path() {
		Local::canonicalize(path).await.map(Into::into)
	} else {
		Err(io::Error::new(io::ErrorKind::Unsupported, "Unsupported filesystem"))
	}
}

#[inline]
pub async fn copy<'a, U, V>(from: U, to: V, cha: Cha) -> io::Result<u64>
where
	U: Into<Url<'a>>,
	V: Into<Url<'a>>,
{
	if let (Some(from), Some(to)) = (from.into().as_path(), to.into().as_path()) {
		Local::copy(from, to, cha).await
	} else {
		Err(io::Error::new(io::ErrorKind::Unsupported, "Unsupported filesystem"))
	}
}

#[inline]
pub async fn create<'a, U>(url: U) -> io::Result<RwFile>
where
	U: Into<Url<'a>>,
{
	if let Some(path) = url.into().as_path() {
		Local::create(path).await.map(Into::into)
	} else {
		Err(io::Error::new(io::ErrorKind::Unsupported, "Unsupported filesystem"))
	}
}

#[inline]
pub async fn create_dir<'a, U>(url: U) -> io::Result<()>
where
	U: Into<Url<'a>>,
{
	if let Some(path) = url.into().as_path() {
		Local::create_dir(path).await
	} else {
		Err(io::Error::new(io::ErrorKind::Unsupported, "Unsupported filesystem"))
	}
}

#[inline]
pub async fn create_dir_all<'a, U>(url: U) -> io::Result<()>
where
	U: Into<Url<'a>>,
{
	if let Some(path) = url.into().as_path() {
		Local::create_dir_all(path).await
	} else {
		Err(io::Error::new(io::ErrorKind::Unsupported, "Unsupported filesystem"))
	}
}

#[inline]
pub async fn hard_link<'a, U, V>(original: U, link: V) -> io::Result<()>
where
	U: Into<Url<'a>>,
	V: Into<Url<'a>>,
{
	if let (Some(original), Some(link)) = (original.into().as_path(), link.into().as_path()) {
		Local::hard_link(original, link).await
	} else {
		Err(io::Error::new(io::ErrorKind::Unsupported, "Unsupported filesystem"))
	}
}

#[inline]
pub async fn identical<'a, U, V>(a: U, b: V) -> io::Result<bool>
where
	U: Into<Url<'a>>,
	V: Into<Url<'a>>,
{
	if let (Some(a), Some(b)) = (a.into().as_path(), b.into().as_path()) {
		local::identical(a, b).await
	} else {
		Err(io::Error::new(io::ErrorKind::Unsupported, "Unsupported filesystem"))
	}
}

#[inline]
pub async fn metadata<'a, U>(url: U) -> io::Result<std::fs::Metadata>
where
	U: Into<Url<'a>>,
{
	if let Some(path) = url.into().as_path() {
		Local::metadata(path).await
	} else {
		Err(io::Error::new(io::ErrorKind::Unsupported, "Unsupported filesystem"))
	}
}

#[inline]
pub async fn must_identical<'a, U, V>(a: U, b: V) -> bool
where
	U: Into<Url<'a>>,
	V: Into<Url<'a>>,
{
	identical(a, b).await.unwrap_or(false)
}

#[inline]
pub async fn open<'a, U>(url: U) -> io::Result<RwFile>
where
	U: Into<Url<'a>>,
{
	if let Some(path) = url.into().as_path() {
		Local::open(path).await.map(Into::into)
	} else {
		Err(io::Error::new(io::ErrorKind::Unsupported, "Unsupported filesystem"))
	}
}

#[inline]
pub async fn read_dir<'a, U>(url: U) -> io::Result<ReadDir>
where
	U: Into<Url<'a>>,
{
	if let Some(path) = url.into().as_path() {
		Local::read_dir(path).await.map(Into::into)
	} else {
		Err(io::Error::new(io::ErrorKind::Unsupported, "Unsupported filesystem"))
	}
}

#[inline]
pub fn read_dir_sync<'a, U>(url: U) -> io::Result<ReadDirSync>
where
	U: Into<Url<'a>>,
{
	if let Some(path) = url.into().as_path() {
		Local::read_dir_sync(path).map(Into::into)
	} else {
		Err(io::Error::new(io::ErrorKind::Unsupported, "Unsupported filesystem"))
	}
}

#[inline]
pub async fn read_link<'a, U>(url: U) -> io::Result<PathBuf>
where
	U: Into<Url<'a>>,
{
	if let Some(path) = url.into().as_path() {
		Local::read_link(path).await
	} else {
		Err(io::Error::new(io::ErrorKind::Unsupported, "Unsupported filesystem"))
	}
}

#[inline]
pub async fn remove_dir<'a, U>(url: U) -> io::Result<()>
where
	U: Into<Url<'a>>,
{
	if let Some(path) = url.into().as_path() {
		Local::remove_dir(path).await
	} else {
		Err(io::Error::new(io::ErrorKind::Unsupported, "Unsupported filesystem"))
	}
}

#[inline]
pub async fn remove_dir_all<'a, U>(url: U) -> io::Result<()>
where
	U: Into<Url<'a>>,
{
	if let Some(path) = url.into().as_path() {
		Local::remove_dir_all(path).await
	} else {
		Err(io::Error::new(io::ErrorKind::Unsupported, "Unsupported filesystem"))
	}
}

#[inline]
pub async fn remove_file<'a, U>(url: U) -> io::Result<()>
where
	U: Into<Url<'a>>,
{
	if let Some(path) = url.into().as_path() {
		Local::remove_file(path).await
	} else {
		Err(io::Error::new(io::ErrorKind::Unsupported, "Unsupported filesystem"))
	}
}

#[inline]
pub async fn rename<'a, U, V>(from: U, to: V) -> io::Result<()>
where
	U: Into<Url<'a>>,
	V: Into<Url<'a>>,
{
	if let (Some(from), Some(to)) = (from.into().as_path(), to.into().as_path()) {
		Local::rename(from, to).await
	} else {
		Err(io::Error::new(io::ErrorKind::Unsupported, "Unsupported filesystem"))
	}
}

#[inline]
pub async fn symlink_dir<'a, U>(original: &Path, link: U) -> io::Result<()>
where
	U: Into<Url<'a>>,
{
	if let Some(link) = link.into().as_path() {
		Local::symlink_dir(original, link).await
	} else {
		Err(io::Error::new(io::ErrorKind::Unsupported, "Unsupported filesystem"))
	}
}

#[inline]
pub async fn symlink_file<'a, U>(original: &Path, link: U) -> io::Result<()>
where
	U: Into<Url<'a>>,
{
	if let Some(link) = link.into().as_path() {
		Local::symlink_file(original, link).await
	} else {
		Err(io::Error::new(io::ErrorKind::Unsupported, "Unsupported filesystem"))
	}
}

#[inline]
pub async fn symlink_metadata<'a, U>(url: U) -> io::Result<std::fs::Metadata>
where
	U: Into<Url<'a>>,
{
	if let Some(path) = url.into().as_path() {
		Local::symlink_metadata(path).await
	} else {
		Err(io::Error::new(io::ErrorKind::Unsupported, "Unsupported filesystem"))
	}
}

#[inline]
pub fn symlink_metadata_sync<'a, U>(url: U) -> io::Result<std::fs::Metadata>
where
	U: Into<Url<'a>>,
{
	if let Some(path) = url.into().as_path() {
		Local::symlink_metadata_sync(path)
	} else {
		Err(io::Error::new(io::ErrorKind::Unsupported, "Unsupported filesystem"))
	}
}

#[inline]
pub async fn trash<'a, U>(url: U) -> io::Result<()>
where
	U: Into<Url<'a>>,
{
	if let Some(path) = url.into().as_path() {
		Local::trash(path).await
	} else {
		Err(io::Error::new(io::ErrorKind::Unsupported, "Unsupported filesystem"))
	}
}

#[inline]
pub async fn write<'a, U, C>(url: U, contents: C) -> io::Result<()>
where
	U: Into<Url<'a>>,
	C: AsRef<[u8]>,
{
	if let Some(path) = url.into().as_path() {
		Local::write(path, contents).await
	} else {
		Err(io::Error::new(io::ErrorKind::Unsupported, "Unsupported filesystem"))
	}
}
