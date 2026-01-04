use std::{io, sync::Arc};

use yazi_shared::url::UrlBuf;

use crate::provider::DirReader;

pub enum ReadDir {
	Regular(tokio::fs::ReadDir),
	Others { reader: tokio::fs::ReadDir, dir: Arc<UrlBuf> },
}

impl DirReader for ReadDir {
	type Entry = super::DirEntry;

	async fn next(&mut self) -> io::Result<Option<Self::Entry>> {
		Ok(match self {
			Self::Regular(reader) => reader.next_entry().await?.map(Self::Entry::Regular),
			Self::Others { reader, dir } => {
				reader.next_entry().await?.map(|entry| Self::Entry::Others { entry, dir: dir.clone() })
			}
		})
	}
}
