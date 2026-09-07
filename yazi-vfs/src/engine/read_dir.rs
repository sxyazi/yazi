use std::io;

use yazi_fs::engine::DirReader;

pub enum ReadDir {
	Local(yazi_fs::engine::local::ReadDir),
	Lua(super::lua::ReadDir),
	Sftp(super::sftp::ReadDir),
}

impl From<yazi_fs::engine::local::ReadDir> for ReadDir {
	fn from(reader: yazi_fs::engine::local::ReadDir) -> Self { Self::Local(reader) }
}

impl From<super::lua::ReadDir> for ReadDir {
	fn from(reader: super::lua::ReadDir) -> Self { Self::Lua(reader) }
}

impl From<super::sftp::ReadDir> for ReadDir {
	fn from(reader: super::sftp::ReadDir) -> Self { Self::Sftp(reader) }
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
