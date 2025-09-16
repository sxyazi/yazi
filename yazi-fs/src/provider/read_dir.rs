use std::io;

use super::DirEntry;
use crate::provider::DirReader;

pub enum ReadDir {
	Local(super::local::ReadDir),
}

impl From<super::local::ReadDir> for ReadDir {
	fn from(value: super::local::ReadDir) -> Self { Self::Local(value) }
}

impl ReadDir {
	pub async fn next_entry(&mut self) -> io::Result<Option<DirEntry>> {
		match self {
			Self::Local(local) => local.next().await.map(|entry| entry.map(Into::into)),
		}
	}
}
