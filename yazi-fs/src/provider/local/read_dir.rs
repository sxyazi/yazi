use std::io;

use super::DirEntry;

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
