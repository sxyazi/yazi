use std::{borrow::Cow, ffi::OsStr, io, path::PathBuf};

use crate::{cha::Cha, provider::{DirReader, FileHolder}};

pub struct ReadDir(pub(super) yazi_sftp::fs::ReadDir);

impl DirReader for ReadDir {
	type Entry<'a> = DirEntry<'a>;

	async fn next(&mut self) -> io::Result<Option<Self::Entry<'_>>> {
		Ok(self.0.next().await?.map(DirEntry))
	}
}

// --- Entry
pub struct DirEntry<'a>(yazi_sftp::fs::DirEntry<'a>);

impl FileHolder for DirEntry<'_> {
	fn path(&self) -> PathBuf { self.0.path() }

	fn name(&self) -> Cow<'_, OsStr> { self.0.name() }

	async fn metadata(&self) -> io::Result<Cha> { todo!() }

	async fn file_type(&self) -> io::Result<std::fs::FileType> { todo!() }
}
