use std::io;

use super::DirEntry;

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
