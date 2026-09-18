use std::io;

use yazi_shared::{path::PathBufDyn, strand::StrandCow, url::{UrlBuf, UrlLike}};

use crate::{engine::FileHolder, file::{File, FileExtra}, stat::{Stat, StatType}};

pub struct DirEntry(pub(super) tokio::fs::DirEntry);

impl FileHolder for DirEntry {
	async fn file(&self) -> io::Result<File> {
		let lstat = self.metadata().await?;
		let url = self.url();

		let (mut followed, mut link_to) = (None, None);
		if lstat.is_link() {
			let path = url.as_local().expect("local entry path");
			let name = path.file_name().unwrap_or_default();
			followed = tokio::fs::metadata(path).await.ok().map(|m| Stat::new(name, m));
			link_to = tokio::fs::read_link(path).await.ok().map(Into::into);
		}

		Ok(File { url, stat: lstat.follow(followed), extra: FileExtra::new(lstat, link_to, None) })
	}

	async fn file_type(&self) -> io::Result<StatType> { self.0.file_type().await.map(Into::into) }

	async fn metadata(&self) -> io::Result<Stat> {
		let meta = self.0.metadata().await?;

		Ok(Stat::new(self.name(), meta)) // TODO: use `file_name_os_str` when stabilized
	}

	fn name(&self) -> StrandCow<'_> { self.0.file_name().into() }

	fn path(&self) -> PathBufDyn { self.0.path().into() }

	fn url(&self) -> UrlBuf { self.0.path().into() }
}
