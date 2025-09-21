use std::{io, path::Path};

use tokio::io::AsyncWriteExt;
use yazi_fs::{ok_or_not_found, provider::{FileBuilder, Provider, local::{Gate, Local}}};

#[inline]
pub async fn must_exists(path: impl AsRef<Path>) -> bool {
	Local.symlink_metadata(path).await.is_ok()
}

#[inline]
pub async fn maybe_exists(path: impl AsRef<Path>) -> bool {
	match Local.symlink_metadata(path).await {
		Ok(_) => true,
		Err(e) => e.kind() != std::io::ErrorKind::NotFound,
	}
}

pub async fn copy_and_seal(from: &Path, to: &Path) -> io::Result<()> {
	let b = Local.read(from).await?;
	ok_or_not_found(remove_sealed(to).await)?;

	let mut file = Gate::default().create_new(true).write(true).truncate(true).open(to).await?;
	file.write_all(&b).await?;

	let mut perm = file.metadata().await?.permissions();
	perm.set_readonly(true);
	file.set_permissions(perm).await?;

	Ok(())
}

// TODO: use `yazi_fs` instead of `tokio::fs`
pub async fn remove_sealed(p: &Path) -> io::Result<()> {
	#[cfg(windows)]
	{
		let mut perm = tokio::fs::metadata(p).await?.permissions();
		perm.set_readonly(false);
		tokio::fs::set_permissions(p, perm).await?;
	}

	Local.remove_file(p).await
}
