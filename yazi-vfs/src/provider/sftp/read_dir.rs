use std::{io, sync::Arc};

use yazi_fs::provider::{DirReader, FileHolder};
use yazi_shared::{path::PathBufDyn, strand::StrandCow, url::{UrlBuf, UrlLike}};

use super::{Cha, ChaMode};

pub struct ReadDir {
	pub(super) dir:    Arc<UrlBuf>,
	pub(super) reader: yazi_sftp::fs::ReadDir,
}

impl DirReader for ReadDir {
	type Entry = DirEntry;

	async fn next(&mut self) -> io::Result<Option<Self::Entry>> {
		Ok(self.reader.next().await?.map(|entry| DirEntry { dir: self.dir.clone(), entry }))
	}
}

// --- Entry
pub struct DirEntry {
	dir:   Arc<UrlBuf>,
	entry: yazi_sftp::fs::DirEntry,
}

impl FileHolder for DirEntry {
	async fn file_type(&self) -> io::Result<yazi_fs::cha::ChaType> {
		Ok(ChaMode::try_from(self.entry.attrs())?.0.into())
	}

	async fn metadata(&self) -> io::Result<yazi_fs::cha::Cha> { Ok(Cha::try_from(&self.entry)?.0) }

	fn name(&self) -> StrandCow<'_> { self.entry.name().into() }

	fn path(&self) -> PathBufDyn { self.entry.path().into() }

	fn url(&self) -> UrlBuf {
		self.dir.try_join(self.entry.name()).expect("entry name is a valid component of the SFTP URL")
	}
}
