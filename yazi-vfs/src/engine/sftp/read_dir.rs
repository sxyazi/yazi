use std::{io, sync::Arc};

use yazi_fs::{engine::{DirReader, FileHolder}, file::File};
use yazi_shared::{path::PathBufDyn, strand::StrandCow, url::{UrlBuf, UrlLike}};

use super::{Cha, ChaMode};
use crate::VfsFile;

pub struct ReadDir {
	pub(super) dir:    Arc<UrlBuf>,
	pub(super) reader: yazi_sftp::fs::ReadDir,
}

impl DirReader for ReadDir {
	type Entry = DirEntry;

	async fn next(&mut self) -> io::Result<Option<Self::Entry>> {
		Ok(self.reader.next().await?.map(|dent| DirEntry { dir: self.dir.clone(), dent }))
	}
}

// --- Entry
pub struct DirEntry {
	dir:  Arc<UrlBuf>,
	dent: yazi_sftp::fs::DirEntry,
}

impl FileHolder for DirEntry {
	async fn file(&self) -> io::Result<File> {
		let cha = self.metadata().await?;
		Ok(File::from_follow(self.url(), cha).await)
	}

	async fn file_type(&self) -> io::Result<yazi_fs::cha::ChaType> {
		Ok(ChaMode::try_from(self.dent.attrs())?.0.into())
	}

	async fn metadata(&self) -> io::Result<yazi_fs::cha::Cha> { Ok(Cha::try_from(&self.dent)?.0) }

	fn name(&self) -> StrandCow<'_> { self.dent.name().into() }

	fn path(&self) -> PathBufDyn { self.dent.path().into() }

	fn url(&self) -> UrlBuf {
		self.dir.try_join(self.dent.name()).expect("entry name is a valid component of the SFTP URL")
	}
}
