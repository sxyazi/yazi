use std::io;

use crate::provider::DirReader;

pub struct ReadDir(pub(super) tokio::fs::ReadDir);

impl DirReader for ReadDir {
	type Entry<'a> = super::DirEntry;

	async fn next(&mut self) -> io::Result<Option<Self::Entry<'_>>> {
		self.0.next_entry().await.map(|entry| entry.map(super::DirEntry))
	}
}
