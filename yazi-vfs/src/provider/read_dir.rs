use std::{io, sync::Arc};

use yazi_fs::provider::DirReader;
use yazi_shared::url::UrlBuf;

pub enum ReadDir {
	Regular(yazi_fs::provider::local::ReadDir),
	Search((Arc<UrlBuf>, yazi_fs::provider::local::ReadDir)),
	Sftp((Arc<UrlBuf>, super::sftp::ReadDir)),
}

impl DirReader for ReadDir {
	type Entry = super::DirEntry;

	async fn next(&mut self) -> io::Result<Option<Self::Entry>> {
		Ok(match self {
			Self::Regular(reader) => reader.next().await?.map(Self::Entry::Regular),
			Self::Search((dir, reader)) => {
				reader.next().await?.map(|ent| Self::Entry::Search((dir.clone(), ent)))
			}
			Self::Sftp((dir, reader)) => {
				reader.next().await?.map(|ent| Self::Entry::Sftp((dir.clone(), ent)))
			}
		})
	}
}
