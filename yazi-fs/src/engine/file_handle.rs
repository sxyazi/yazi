use std::{fs::Permissions, io, ops::Deref, sync::Arc};

use tokio::sync::OnceCell;

use super::{Attrs, FileCursor};

pub struct FileHandle {
	pub(super) tokio: tokio::fs::File,
	std:              OnceCell<Arc<std::fs::File>>,
}

impl Deref for FileHandle {
	type Target = tokio::fs::File;

	fn deref(&self) -> &Self::Target { &self.tokio }
}

impl FileHandle {
	pub(super) fn new(tokio: tokio::fs::File) -> Self { Self { tokio, std: OnceCell::new() } }

	pub async fn cursor(&self, offset: u64) -> io::Result<FileCursor> {
		let file = self
			.std
			.get_or_try_init(async || -> io::Result<_> {
				Ok(Arc::new(self.tokio.try_clone().await?.into_std().await))
			})
			.await?;

		Ok(FileCursor::new(file.clone(), offset))
	}

	pub async fn permissions(&self) -> io::Result<Permissions> {
		self.tokio.metadata().await.map(|meta| meta.permissions())
	}

	pub async fn set_attrs(&self, attrs: Attrs) -> io::Result<()> {
		let (perm, times) = (attrs.try_into(), attrs.try_into());
		if perm.is_err() && times.is_err() {
			return Ok(());
		}

		let std = self.tokio.try_clone().await?.into_std().await;
		tokio::task::spawn_blocking(move || {
			perm.map(|p| std.set_permissions(p)).ok();
			times.map(|t| std.set_times(t)).ok();
		})
		.await?;

		Ok(())
	}
}
