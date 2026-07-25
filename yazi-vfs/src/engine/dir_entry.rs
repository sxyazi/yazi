use std::io;

use yazi_fs::{cha::{Cha, ChaType}, engine::FileHolder, file::File};
use yazi_shared::{path::PathBufDyn, strand::StrandCow, url::UrlBuf};

pub enum DirEntry {
	Local(yazi_fs::engine::local::DirEntry),
	Lua(super::lua::DirEntry),
	Sftp(super::sftp::DirEntry),
}

impl FileHolder for DirEntry {
	async fn file(&self) -> io::Result<File> {
		match self {
			Self::Local(dent) => dent.file().await,
			Self::Lua(dent) => dent.file().await,
			Self::Sftp(dent) => dent.file().await,
		}
	}

	async fn file_type(&self) -> io::Result<ChaType> {
		match self {
			Self::Local(dent) => dent.file_type().await,
			Self::Lua(dent) => dent.file_type().await,
			Self::Sftp(dent) => dent.file_type().await,
		}
	}

	async fn metadata(&self) -> io::Result<Cha> {
		match self {
			Self::Local(dent) => dent.metadata().await,
			Self::Lua(dent) => dent.metadata().await,
			Self::Sftp(dent) => dent.metadata().await,
		}
	}

	fn name(&self) -> StrandCow<'_> {
		match self {
			Self::Local(dent) => dent.name(),
			Self::Lua(dent) => dent.name(),
			Self::Sftp(dent) => dent.name(),
		}
	}

	fn path(&self) -> PathBufDyn {
		match self {
			Self::Local(dent) => dent.path(),
			Self::Lua(dent) => dent.path(),
			Self::Sftp(dent) => dent.path(),
		}
	}

	fn url(&self) -> UrlBuf {
		match self {
			Self::Local(dent) => dent.url(),
			Self::Lua(dent) => dent.url(),
			Self::Sftp(dent) => dent.url(),
		}
	}
}
