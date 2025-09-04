use std::io;

use super::{DirEntry, DirEntrySync};

pub struct ReadDir(tokio::fs::ReadDir);

impl From<tokio::fs::ReadDir> for ReadDir {
	fn from(value: tokio::fs::ReadDir) -> Self { Self(value) }
}

impl From<ReadDir> for crate::provider::ReadDir {
	fn from(value: ReadDir) -> Self { Self::Local(value) }
}

impl ReadDir {
	pub async fn next_entry(&mut self) -> io::Result<Option<DirEntry>> {
		self.0.next_entry().await.map(|entry| entry.map(Into::into))
	}
}

// --- ReadDirSync
pub struct ReadDirSync(std::fs::ReadDir);

impl From<std::fs::ReadDir> for ReadDirSync {
	fn from(value: std::fs::ReadDir) -> Self { Self(value) }
}

impl From<ReadDirSync> for crate::provider::ReadDirSync {
	fn from(value: ReadDirSync) -> Self { Self::Local(value) }
}

impl Iterator for ReadDirSync {
	type Item = io::Result<DirEntrySync>;

	fn next(&mut self) -> Option<io::Result<DirEntrySync>> {
		self.0.next().map(|result| result.map(Into::into))
	}
}
