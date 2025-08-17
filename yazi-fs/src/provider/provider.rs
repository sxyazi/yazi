use std::io;

use yazi_shared::url::{Url, UrlBuf};

use crate::provider::{ReadDir, ReadDirSync, RwFile, local::Local};

#[inline]
pub async fn canonicalize<'a>(url: impl Into<Url<'a>>) -> io::Result<UrlBuf> {
	if let Some(path) = url.into().as_path() {
		Local::canonicalize(path).await.map(Into::into)
	} else {
		Err(io::Error::new(io::ErrorKind::Unsupported, "Unsupported filesystem"))
	}
}

#[inline]
pub async fn create<'a>(url: impl Into<Url<'a>>) -> io::Result<RwFile> {
	if let Some(path) = url.into().as_path() {
		Local::create(path).await.map(Into::into)
	} else {
		Err(io::Error::new(io::ErrorKind::Unsupported, "Unsupported filesystem"))
	}
}

#[inline]
pub async fn create_dir<'a>(url: impl Into<Url<'a>>) -> io::Result<()> {
	if let Some(path) = url.into().as_path() {
		Local::create_dir(path).await
	} else {
		Err(io::Error::new(io::ErrorKind::Unsupported, "Unsupported filesystem"))
	}
}

#[inline]
pub async fn create_dir_all<'a>(url: impl Into<Url<'a>>) -> io::Result<()> {
	if let Some(path) = url.into().as_path() {
		Local::create_dir_all(path).await
	} else {
		Err(io::Error::new(io::ErrorKind::Unsupported, "Unsupported filesystem"))
	}
}

#[inline]
pub async fn hard_link<'a>(
	original: impl Into<Url<'a>>,
	link: impl Into<Url<'a>>,
) -> io::Result<()> {
	if let (Some(original), Some(link)) = (original.into().as_path(), link.into().as_path()) {
		Local::hard_link(original, link).await
	} else {
		Err(io::Error::new(io::ErrorKind::Unsupported, "Unsupported filesystem"))
	}
}

#[inline]
pub async fn metadata<'a>(url: impl Into<Url<'a>>) -> io::Result<std::fs::Metadata> {
	if let Some(path) = url.into().as_path() {
		Local::metadata(path).await
	} else {
		Err(io::Error::new(io::ErrorKind::Unsupported, "Unsupported filesystem"))
	}
}

#[inline]
pub async fn open<'a>(url: impl Into<Url<'a>>) -> io::Result<RwFile> {
	if let Some(path) = url.into().as_path() {
		Local::open(path).await.map(Into::into)
	} else {
		Err(io::Error::new(io::ErrorKind::Unsupported, "Unsupported filesystem"))
	}
}

#[inline]
pub async fn read_dir<'a>(url: impl Into<Url<'a>>) -> io::Result<ReadDir> {
	if let Some(path) = url.into().as_path() {
		Local::read_dir(path).await.map(Into::into)
	} else {
		Err(io::Error::new(io::ErrorKind::Unsupported, "Unsupported filesystem"))
	}
}

#[inline]
pub fn read_dir_sync<'a>(url: impl Into<Url<'a>>) -> io::Result<ReadDirSync> {
	if let Some(path) = url.into().as_path() {
		Local::read_dir_sync(path).map(Into::into)
	} else {
		Err(io::Error::new(io::ErrorKind::Unsupported, "Unsupported filesystem"))
	}
}

#[inline]
pub async fn read_link<'a>(url: impl Into<Url<'a>>) -> io::Result<UrlBuf> {
	if let Some(path) = url.into().as_path() {
		Local::read_link(path).await.map(Into::into)
	} else {
		Err(io::Error::new(io::ErrorKind::Unsupported, "Unsupported filesystem"))
	}
}

#[inline]
pub async fn remove_dir<'a>(url: impl Into<Url<'a>>) -> io::Result<()> {
	if let Some(path) = url.into().as_path() {
		Local::remove_dir(path).await
	} else {
		Err(io::Error::new(io::ErrorKind::Unsupported, "Unsupported filesystem"))
	}
}

#[inline]
pub async fn remove_dir_all<'a>(url: impl Into<Url<'a>>) -> io::Result<()> {
	if let Some(path) = url.into().as_path() {
		Local::remove_dir_all(path).await
	} else {
		Err(io::Error::new(io::ErrorKind::Unsupported, "Unsupported filesystem"))
	}
}

#[inline]
pub async fn remove_file<'a>(url: impl Into<Url<'a>>) -> io::Result<()> {
	if let Some(path) = url.into().as_path() {
		Local::remove_file(path).await
	} else {
		Err(io::Error::new(io::ErrorKind::Unsupported, "Unsupported filesystem"))
	}
}

#[inline]
pub async fn rename<'a>(from: impl Into<Url<'a>>, to: impl Into<Url<'a>>) -> io::Result<()> {
	if let (Some(from), Some(to)) = (from.into().as_path(), to.into().as_path()) {
		Local::rename(from, to).await
	} else {
		Err(io::Error::new(io::ErrorKind::Unsupported, "Unsupported filesystem"))
	}
}

#[inline]
pub async fn symlink_dir<'a>(
	original: impl Into<Url<'a>>,
	link: impl Into<Url<'a>>,
) -> io::Result<()> {
	if let (Some(original), Some(link)) = (original.into().as_path(), link.into().as_path()) {
		Local::symlink_dir(original, link).await
	} else {
		Err(io::Error::new(io::ErrorKind::Unsupported, "Unsupported filesystem"))
	}
}

#[inline]
pub async fn symlink_file<'a>(
	original: impl Into<Url<'a>>,
	link: impl Into<Url<'a>>,
) -> io::Result<()> {
	if let (Some(original), Some(link)) = (original.into().as_path(), link.into().as_path()) {
		Local::symlink_file(original, link).await
	} else {
		Err(io::Error::new(io::ErrorKind::Unsupported, "Unsupported filesystem"))
	}
}

#[inline]
pub async fn symlink_metadata<'a>(url: impl Into<Url<'a>>) -> io::Result<std::fs::Metadata> {
	if let Some(path) = url.into().as_path() {
		Local::symlink_metadata(path).await
	} else {
		Err(io::Error::new(io::ErrorKind::Unsupported, "Unsupported filesystem"))
	}
}

pub fn symlink_metadata_sync<'a>(url: impl Into<Url<'a>>) -> io::Result<std::fs::Metadata> {
	if let Some(path) = url.into().as_path() {
		Local::symlink_metadata_sync(path)
	} else {
		Err(io::Error::new(io::ErrorKind::Unsupported, "Unsupported filesystem"))
	}
}

#[inline]
pub async fn write<'a>(url: impl Into<Url<'a>>, contents: impl AsRef<[u8]>) -> io::Result<()> {
	if let Some(path) = url.into().as_path() {
		Local::write(path, contents).await
	} else {
		Err(io::Error::new(io::ErrorKind::Unsupported, "Unsupported filesystem"))
	}
}
