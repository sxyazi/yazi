use std::{ffi::OsString, io};

use yazi_shared::url::UrlBuf;

pub enum DirEntry {
	Local(super::local::DirEntry),
}

impl DirEntry {
	#[must_use]
	pub fn url(&self) -> UrlBuf {
		match self {
			DirEntry::Local(local) => local.url(),
		}
	}

	#[must_use]
	pub fn file_name(&self) -> OsString {
		match self {
			DirEntry::Local(local) => local.file_name(),
		}
	}

	pub async fn metadata(&self) -> io::Result<std::fs::Metadata> {
		match self {
			DirEntry::Local(local) => local.metadata().await,
		}
	}

	pub async fn file_type(&self) -> io::Result<std::fs::FileType> {
		match self {
			DirEntry::Local(local) => local.file_type().await,
		}
	}
}

// --- DirEntrySync
pub enum DirEntrySync {
	Local(super::local::DirEntrySync),
}

impl DirEntrySync {
	#[must_use]
	pub fn url(&self) -> UrlBuf {
		match self {
			DirEntrySync::Local(local) => local.url(),
		}
	}

	#[must_use]
	pub fn file_name(&self) -> OsString {
		match self {
			DirEntrySync::Local(local) => local.file_name(),
		}
	}

	pub fn metadata(&self) -> io::Result<std::fs::Metadata> {
		match self {
			DirEntrySync::Local(local) => local.metadata(),
		}
	}

	pub fn file_type(&self) -> io::Result<std::fs::FileType> {
		match self {
			DirEntrySync::Local(local) => local.file_type(),
		}
	}
}
