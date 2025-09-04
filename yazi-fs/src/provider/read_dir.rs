use std::io;

use super::{DirEntry, DirEntrySync};

pub enum ReadDir {
	Local(super::local::ReadDir),
}

impl ReadDir {
	pub async fn next_entry(&mut self) -> io::Result<Option<DirEntry>> {
		match self {
			Self::Local(local) => local.next_entry().await.map(|entry| entry.map(Into::into)),
		}
	}
}

// --- ReadDirSync
pub enum ReadDirSync {
	Local(super::local::ReadDirSync),
}

impl Iterator for ReadDirSync {
	type Item = io::Result<DirEntrySync>;

	fn next(&mut self) -> Option<io::Result<DirEntrySync>> {
		match self {
			Self::Local(local) => local.next().map(|result| result.map(Into::into)),
		}
	}
}
