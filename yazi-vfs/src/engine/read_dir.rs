use std::io;

use yazi_fs::engine::DirReader;

pub enum ReadDir {
	Local(yazi_fs::engine::local::ReadDir),
	Lua(super::lua::ReadDir),
	Sftp(super::sftp::ReadDir),
}

impl DirReader for ReadDir {
	type Entry = super::DirEntry;

	async fn next(&mut self) -> io::Result<Option<Self::Entry>> {
		Ok(match self {
			Self::Local(reader) => reader.next().await?.map(Self::Entry::Local),
			Self::Lua(reader) => reader.next().await?.map(Self::Entry::Lua),
			Self::Sftp(reader) => reader.next().await?.map(Self::Entry::Sftp),
		})
	}
}
