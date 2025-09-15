use std::{ffi::OsString, io};

use yazi_shared::url::UrlBuf;

pub enum DirEntry {
	Local(super::local::DirEntry),
}

impl DirEntry {
	#[must_use]
	pub fn url(&self) -> UrlBuf {
		match self {
			Self::Local(local) => local.url(),
		}
	}

	#[must_use]
	pub fn file_name(&self) -> OsString {
		match self {
			Self::Local(local) => local.file_name(),
		}
	}

	pub async fn metadata(&self) -> io::Result<std::fs::Metadata> {
		match self {
			Self::Local(local) => local.metadata().await,
		}
	}

	pub async fn file_type(&self) -> io::Result<std::fs::FileType> {
		match self {
			Self::Local(local) => local.file_type().await,
		}
	}
}
