use std::io;

use crate::engine::DirReader;

pub struct ReadDir(pub(super) tokio::fs::ReadDir);

impl DirReader for ReadDir {
	type Entry = super::DirEntry;

	async fn next(&mut self) -> io::Result<Option<Self::Entry>> {
		Ok(self.0.next_entry().await?.map(super::DirEntry))
	}
}
