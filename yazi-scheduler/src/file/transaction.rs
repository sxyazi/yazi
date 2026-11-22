use std::{hash::BuildHasher, io::{self, ErrorKind}};

use yazi_fs::{engine::Attrs, stat::StatMode};
use yazi_macro::ok_or_not_found;
use yazi_shared::{timestamp_us, url::{AsUrl, UrlBuf, UrlLike}};
use yazi_vfs::{engine::{self, RwFile}, unique_file};

pub(super) struct Transaction;

impl Transaction {
	pub(super) async fn tmp(url: impl AsUrl) -> io::Result<UrlBuf> {
		let url = url.as_url();
		let Some(parent) = url.parent() else {
			Err(io::Error::new(ErrorKind::InvalidInput, "Url has no parent"))?
		};

		let h = foldhash::fast::FixedState::default().hash_one((url, timestamp_us()));
		unique_file(parent.try_join(format!(".{h:x}.yazitmp"))?).await.map(RwFile::into_url)
	}

	pub(super) async fn prepare(from: &UrlBuf, target: &mut UrlBuf) -> io::Result<()> {
		let url = target.as_url();
		let stat = ok_or_not_found!(engine::lstat(url).await, return Ok(()));

		if stat.is_indirect() && engine::same_entry(from, url).await? {
			return Err(io::Error::new(ErrorKind::InvalidInput, "Cannot copy a file onto itself"));
		} else if stat.is_indirect() {
			*target = engine::rename(Self::tmp(url).await?, url).await?;
			return Ok(());
		}

		// The local copier checks identity itself.
		let check_identity = !from.auth().is_local()
			|| !url.auth().is_local()
			|| !from.is_regular() && engine::capabilities(from).await?.same_file;

		if check_identity && engine::same_file(from, url).await? {
			return Err(io::Error::new(ErrorKind::InvalidInput, "Cannot copy a file onto itself"));
		} else if !stat.contains(StatMode::U_WRITE) {
			engine::set_attrs(url, Attrs::mode(stat.mode | StatMode::U_WRITE)).await?;
		}

		Ok(())
	}
}
