use std::{borrow::Cow, ffi::OsStr, io};

use yazi_shared::url::UrlBuf;

use crate::{cha::Cha, provider::FileHolder};

pub enum DirEntry {
	Local(super::local::DirEntry),
}

impl From<super::local::DirEntry> for DirEntry {
	fn from(value: super::local::DirEntry) -> Self { Self::Local(value) }
}

impl DirEntry {
	#[must_use]
	pub fn url(&self) -> UrlBuf {
		match self {
			Self::Local(local) => local.path().into(),
		}
	}

	#[must_use]
	pub fn name(&self) -> Cow<'_, OsStr> {
		match self {
			Self::Local(local) => local.name(),
		}
	}

	pub async fn metadata(&self) -> io::Result<Cha> {
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
