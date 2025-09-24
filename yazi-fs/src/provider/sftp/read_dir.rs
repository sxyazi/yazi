use std::{borrow::Cow, ffi::OsStr, io, path::PathBuf};

use crate::{cha::{Cha, ChaMode, ChaType}, provider::{DirReader, FileHolder}};

pub struct ReadDir(pub(super) yazi_sftp::fs::ReadDir);

impl DirReader for ReadDir {
	type Entry = DirEntry;

	async fn next(&mut self) -> io::Result<Option<Self::Entry>> {
		Ok(self.0.next().await?.map(DirEntry))
	}
}

// --- Entry
pub struct DirEntry(yazi_sftp::fs::DirEntry);

impl FileHolder for DirEntry {
	fn path(&self) -> PathBuf { self.0.path() }

	fn name(&self) -> Cow<'_, OsStr> { self.0.name() }

	async fn metadata(&self) -> io::Result<Cha> { Cha::try_from(&self.0) }

	async fn file_type(&self) -> io::Result<ChaType> {
		ChaMode::try_from(self.0.attrs()).map(Into::into)
	}
}
