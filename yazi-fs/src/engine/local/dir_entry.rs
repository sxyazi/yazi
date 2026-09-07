use std::io;

use yazi_shared::{path::PathBufDyn, strand::StrandCow, url::{UrlBuf, UrlLike}};

use crate::{cha::{Cha, ChaType}, engine::FileHolder, file::{File, FileExtra}};

pub struct DirEntry(pub(super) tokio::fs::DirEntry);

impl FileHolder for DirEntry {
	async fn file(&self) -> io::Result<File> {
		let cha = self.metadata().await?;
		let url = self.url();

		let (mut followed, mut link_to) = (None, None);
		if cha.is_link() {
			let path = url.as_local().expect("local entry path");
			let name = path.file_name().unwrap_or_default();
			followed = tokio::fs::metadata(path).await.ok().map(|m| Cha::new(name, m));
			link_to = tokio::fs::read_link(path).await.ok().map(Into::into);
		}

		Ok(File { url, cha: cha.follow(followed), extra: FileExtra::new(link_to, None) })
	}

	async fn file_type(&self) -> io::Result<ChaType> { self.0.file_type().await.map(Into::into) }

	async fn metadata(&self) -> io::Result<Cha> {
		let meta = self.0.metadata().await?;

		Ok(Cha::new(self.name(), meta)) // TODO: use `file_name_os_str` when stabilized
	}

	fn name(&self) -> StrandCow<'_> { self.0.file_name().into() }

	fn path(&self) -> PathBufDyn { self.0.path().into() }

	fn url(&self) -> UrlBuf { self.0.path().into() }
}
