use std::{collections::VecDeque, io, sync::Arc};

use yazi_fs::provider::{DirReader, FileHolder};
use yazi_shared::{path::PathBufDyn, strand::StrandCow, url::{UrlBuf, UrlLike}};

use super::Item;

pub struct ReadDir {
	pub(super) dir:     Arc<UrlBuf>,
	pub(super) entries: VecDeque<Item>,
}

impl DirReader for ReadDir {
	type Entry = DirEntry;

	async fn next(&mut self) -> io::Result<Option<Self::Entry>> {
		Ok(self.entries.pop_front().map(|item| DirEntry { dir: self.dir.clone(), item }))
	}
}

// --- Entry
pub struct DirEntry {
	dir:  Arc<UrlBuf>,
	item: Item,
}

impl FileHolder for DirEntry {
	async fn file_type(&self) -> io::Result<yazi_fs::cha::ChaType> {
		Ok(self.item.cha()?.mode.into())
	}

	async fn metadata(&self) -> io::Result<yazi_fs::cha::Cha> { self.item.cha() }

	fn name(&self) -> StrandCow<'_> { self.item.name.as_str().into() }

	fn path(&self) -> PathBufDyn { self.url().into_loc() }

	fn url(&self) -> UrlBuf {
		self
			.dir
			.try_join(self.item.name.as_str())
			.expect("entry name is a valid component of the rclone URL")
	}
}
