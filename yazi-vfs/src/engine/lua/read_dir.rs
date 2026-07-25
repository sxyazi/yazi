use std::{io, vec};

use yazi_fs::{cha::{Cha, ChaType}, engine::{DirReader, FileHolder}};
use yazi_shared::{path::PathBufDyn, strand::StrandCow, url::{UrlBuf, UrlLike}};

pub struct ReadDir {
	pub(super) files: vec::IntoIter<yazi_fs::file::File>,
}

impl DirReader for ReadDir {
	type Entry = DirEntry;

	async fn next(&mut self) -> io::Result<Option<Self::Entry>> {
		Ok(self.files.next().map(|file| DirEntry { file }))
	}
}

// --- Entry
pub struct DirEntry {
	file: yazi_fs::file::File,
}

impl FileHolder for DirEntry {
	async fn file(&self) -> io::Result<yazi_fs::file::File> { Ok(self.file.clone()) }

	async fn file_type(&self) -> io::Result<ChaType> { Ok(**self.file.cha) }

	async fn metadata(&self) -> io::Result<Cha> { Ok(self.file.cha) }

	fn name(&self) -> StrandCow<'_> { self.file.name().unwrap_or_default().into() }

	fn path(&self) -> PathBufDyn { self.file.url.loc().into() }

	fn url(&self) -> UrlBuf { self.file.url.clone() }
}
