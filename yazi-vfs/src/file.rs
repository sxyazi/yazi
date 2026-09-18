use std::io;

use yazi_fs::{file::{File, FileExtra}, stat::Stat};
use yazi_shared::url::{UrlBuf, UrlCow};

use crate::{VfsStat, engine};

pub trait VfsFile: Sized {
	fn maybe_new<'a>(url: impl Into<UrlCow<'a>>) -> impl Future<Output = io::Result<Option<Self>>>;

	fn from_follow(url: UrlBuf, stat: Stat) -> impl Future<Output = Self>;
}

impl VfsFile for File {
	async fn maybe_new<'a>(url: impl Into<UrlCow<'a>>) -> io::Result<Option<Self>> {
		let url = url.into();
		Ok(match engine::file(&url).await {
			Ok(file) => Some(file),
			Err(e) if e.kind() == io::ErrorKind::NotFound => None,
			Err(e) => return Err(e),
		})
	}

	async fn from_follow(url: UrlBuf, lstat: Stat) -> Self {
		let link_to = if lstat.is_link() { engine::read_link(&url).await.ok() } else { None };
		let stat = Stat::from_follow(&url, lstat).await;

		Self { url, stat, extra: FileExtra::new(lstat, link_to, None) }
	}
}
