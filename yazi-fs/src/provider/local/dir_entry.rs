use std::{borrow::Cow, ffi::OsStr, io, path::PathBuf};

use crate::{cha::{Cha, ChaType}, provider::FileHolder};

pub struct DirEntry(pub(super) tokio::fs::DirEntry);

impl FileHolder for DirEntry {
	fn path(&self) -> PathBuf { self.0.path() }

	fn name(&self) -> Cow<'_, OsStr> { self.0.file_name().into() }

	async fn metadata(&self) -> io::Result<Cha> {
		let name = self.name(); // TODO: use `file_name_os_str` when stabilized
		Ok(Cha::new(&name, self.0.metadata().await?))
	}

	async fn file_type(&self) -> io::Result<ChaType> { self.0.file_type().await.map(Into::into) }
}
