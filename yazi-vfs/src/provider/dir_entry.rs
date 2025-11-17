use std::io;

use yazi_fs::{cha::{Cha, ChaType}, provider::FileHolder};
use yazi_shared::{path::PathBufDyn, strand::StrandCow, url::UrlBuf};

pub enum DirEntry {
	Local(yazi_fs::provider::local::DirEntry),
	Sftp(super::sftp::DirEntry),
}

impl FileHolder for DirEntry {
	async fn file_type(&self) -> io::Result<ChaType> {
		match self {
			Self::Local(entry) => entry.file_type().await,
			Self::Sftp(entry) => entry.file_type().await,
		}
	}

	async fn metadata(&self) -> io::Result<Cha> {
		match self {
			Self::Local(entry) => entry.metadata().await,
			Self::Sftp(entry) => entry.metadata().await,
		}
	}

	fn name(&self) -> StrandCow<'_> {
		match self {
			Self::Local(entry) => entry.name(),
			Self::Sftp(entry) => entry.name(),
		}
	}

	fn path(&self) -> PathBufDyn {
		match self {
			Self::Local(entry) => entry.path(),
			Self::Sftp(entry) => entry.path(),
		}
	}

	fn url(&self) -> UrlBuf {
		match self {
			Self::Local(entry) => entry.url(),
			Self::Sftp(entry) => entry.url(),
		}
	}
}
