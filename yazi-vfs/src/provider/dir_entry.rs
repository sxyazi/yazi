use std::{borrow::Cow, ffi::OsStr, io, sync::Arc};

use yazi_fs::{cha::{Cha, ChaType}, provider::FileHolder};
use yazi_shared::url::{UrlBuf, UrlLike};

pub enum DirEntry {
	Regular(yazi_fs::provider::local::DirEntry),
	Search((Arc<UrlBuf>, yazi_fs::provider::local::DirEntry)),
	Sftp((Arc<UrlBuf>, super::sftp::DirEntry)),
}

impl FileHolder for DirEntry {
	fn path(&self) -> std::path::PathBuf {
		match self {
			Self::Regular(ent) => ent.path(),
			Self::Search((_, ent)) => ent.path(),
			Self::Sftp((_, ent)) => ent.path(),
		}
	}

	fn name(&self) -> Cow<'_, OsStr> {
		match self {
			Self::Regular(ent) => ent.name(),
			Self::Search((_, ent)) => ent.name(),
			Self::Sftp((_, ent)) => ent.name(),
		}
	}

	async fn metadata(&self) -> io::Result<Cha> {
		match self {
			Self::Regular(ent) => ent.metadata().await,
			Self::Search((_, ent)) => ent.metadata().await,
			Self::Sftp((_, ent)) => ent.metadata().await,
		}
	}

	async fn file_type(&self) -> io::Result<ChaType> {
		match self {
			Self::Regular(ent) => ent.file_type().await,
			Self::Search((_, ent)) => ent.file_type().await,
			Self::Sftp((_, ent)) => ent.file_type().await,
		}
	}
}

impl DirEntry {
	#[must_use]
	pub fn url(&self) -> UrlBuf {
		match self {
			Self::Regular(ent) => ent.path().into(),
			Self::Search((dir, ent)) => dir.join(ent.name()),
			Self::Sftp((dir, ent)) => dir.join(ent.name()),
		}
	}
}
