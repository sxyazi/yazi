use std::{io, sync::Arc};

use yazi_shared::{path::PathBufDyn, strand::StrandCow, url::{UrlBuf, UrlLike}};

use crate::{cha::{Cha, ChaType}, provider::FileHolder};

pub enum DirEntry {
	Regular(tokio::fs::DirEntry),
	Others { entry: tokio::fs::DirEntry, dir: Arc<UrlBuf> },
}

impl FileHolder for DirEntry {
	async fn file_type(&self) -> io::Result<ChaType> {
		match self {
			Self::Regular(entry) | Self::Others { entry, .. } => entry.file_type().await.map(Into::into),
		}
	}

	async fn metadata(&self) -> io::Result<Cha> {
		let meta = match self {
			Self::Regular(entry) | Self::Others { entry, .. } => entry.metadata().await?,
		};

		Ok(Cha::new(self.name(), meta)) // TODO: use `file_name_os_str` when stabilized
	}

	fn name(&self) -> StrandCow<'_> {
		match self {
			Self::Regular(entry) | Self::Others { entry, .. } => entry.file_name().into(),
		}
	}

	fn path(&self) -> PathBufDyn {
		match self {
			Self::Regular(entry) | Self::Others { entry, .. } => entry.path().into(),
		}
	}

	fn url(&self) -> UrlBuf {
		match self {
			Self::Regular(entry) => entry.path().into(),
			Self::Others { entry, dir } => {
				dir.try_join(entry.file_name()).expect("entry name is a valid component of the local URL")
			}
		}
	}
}
