use std::{hash::{BuildHasher, Hash, Hasher}, io};

use yazi_fs::{engine::Attrs, stat::StatMode};
use yazi_macro::ok_or_not_found;
use yazi_shared::{timestamp_us, url::{AsUrl, Url, UrlBuf}};
use yazi_vfs::{engine::{self, RwFile}, unique_file};

pub(super) struct Transaction;

impl Transaction {
	pub(super) async fn tmp<U>(url: U) -> io::Result<UrlBuf>
	where
		U: AsUrl,
	{
		Self::tmp_impl(url.as_url()).await
	}

	async fn tmp_impl(url: Url<'_>) -> io::Result<UrlBuf> {
		let Some(parent) = url.parent() else {
			Err(io::Error::new(io::ErrorKind::InvalidInput, "Url has no parent"))?
		};

		let mut h = foldhash::fast::FixedState::default().build_hasher();
		url.hash(&mut h);
		timestamp_us().hash(&mut h);

		unique_file(parent.try_join(format!(".{:x}.yazitmp", h.finish()))?).await.map(RwFile::into_url)
	}

	pub(super) async fn unlink<U, V>(from: U, url: V) -> io::Result<()>
	where
		U: AsUrl,
		V: AsUrl,
	{
		let url = url.as_url();

		let stat = ok_or_not_found!(engine::lstat(url).await, return Ok(()));
		if stat.is_indirect() {
			// Replacing a link that is the source itself would destroy it.
			if Self::same_inode(from, url).await {
				return Err(io::Error::new(io::ErrorKind::InvalidInput, "Destination link is the source"));
			}
			engine::rename(Self::tmp(url).await?, url).await?;
		} else if !stat.contains(StatMode::U_WRITE) {
			engine::set_attrs(url, Attrs::mode(stat.mode | StatMode::U_WRITE)).await?;
		}

		Ok(())
	}

	/// Whether `a` and `b` share an inode, without following a final link. False unless both are local on Unix.
	pub(super) async fn same_inode<U, V>(a: U, b: V) -> bool
	where
		U: AsUrl,
		V: AsUrl,
	{
		#[cfg(unix)]
		if let (Some(a), Some(b)) = (a.as_url().as_local(), b.as_url().as_local()) {
			use std::os::unix::fs::MetadataExt;

			if let (Ok(a), Ok(b)) =
				(tokio::fs::symlink_metadata(a).await, tokio::fs::symlink_metadata(b).await)
			{
				return a.dev() == b.dev() && a.ino() == b.ino();
			}
		}

		_ = (a, b);
		false
	}
}
