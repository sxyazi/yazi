use std::{hash::{BuildHasher, Hash, Hasher}, io};

use yazi_fs::{engine::Attrs, stat::StatMode};
use yazi_macro::ok_or_not_found;
use yazi_shared::{timestamp_us, url::{AsUrl, Url, UrlBuf}};
use yazi_vfs::{engine, unique_file};

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

		unique_file(parent.try_join(format!(".{:x}.%tmp", h.finish()))?, false).await
	}

	pub(super) async fn unlink<U>(url: U) -> io::Result<()>
	where
		U: AsUrl,
	{
		let url = url.as_url();

		let stat = ok_or_not_found!(engine::symlink_metadata(url).await, return Ok(()));
		if stat.is_indirect() {
			engine::rename(Self::tmp(url).await?, url).await?;
		} else if !stat.contains(StatMode::U_WRITE) {
			engine::set_attrs(url, Attrs::mode(stat.mode | StatMode::U_WRITE)).await?;
		}

		Ok(())
	}
}
