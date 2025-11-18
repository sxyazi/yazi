use std::io;

use yazi_fs::provider::DirReader;

pub enum ReadDir {
	Local(yazi_fs::provider::local::ReadDir),
	Sftp(super::sftp::ReadDir),
}

impl DirReader for ReadDir {
	type Entry = super::DirEntry;

	async fn next(&mut self) -> io::Result<Option<Self::Entry>> {
		Ok(match self {
			Self::Local(reader) => reader.next().await?.map(Self::Entry::Local),
			Self::Sftp(reader) => reader.next().await?.map(Self::Entry::Sftp),
		})
	}
}
