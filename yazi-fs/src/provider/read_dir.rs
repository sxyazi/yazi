use std::{io, sync::Arc};

use yazi_shared::url::UrlBuf;

use crate::provider::DirReader;

pub enum ReadDir {
	Regular(super::local::ReadDir),
	Search((Arc<UrlBuf>, super::local::ReadDir)),
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
