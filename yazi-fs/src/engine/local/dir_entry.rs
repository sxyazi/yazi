use std::{io, sync::Arc};

use yazi_shared::{path::PathBufDyn, strand::StrandCow, url::{UrlBuf, UrlLike}};

use crate::{cha::{Cha, ChaType}, engine::FileHolder, file::{File, FileExtra}};

pub enum DirEntry {
	Regular(tokio::fs::DirEntry),
	Others { dent: tokio::fs::DirEntry, dir: Arc<UrlBuf> },
}

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

	async fn file_type(&self) -> io::Result<ChaType> {
		match self {
			Self::Regular(dent) | Self::Others { dent, .. } => dent.file_type().await.map(Into::into),
		}
	}

	async fn metadata(&self) -> io::Result<Cha> {
		let meta = match self {
			Self::Regular(dent) | Self::Others { dent, .. } => dent.metadata().await?,
		};

		Ok(Cha::new(self.name(), meta)) // TODO: use `file_name_os_str` when stabilized
	}

	fn name(&self) -> StrandCow<'_> {
		match self {
			Self::Regular(dent) | Self::Others { dent, .. } => dent.file_name().into(),
		}
	}

	fn path(&self) -> PathBufDyn {
		match self {
			Self::Regular(dent) | Self::Others { dent, .. } => dent.path().into(),
		}
	}

	fn url(&self) -> UrlBuf {
		match self {
			Self::Regular(dent) => dent.path().into(),
			Self::Others { dent, dir } => {
				dir.try_join(dent.file_name()).expect("entry name is a valid component of the local URL")
			}
		}
	}
}
