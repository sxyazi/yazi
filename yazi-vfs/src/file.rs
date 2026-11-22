use std::io;

use yazi_fs::{file::{File, FileExtra}, ok_or_not_found, stat::Stat};
use yazi_shared::url::{UrlBuf, UrlCow};

use crate::{VfsStat, engine};

pub trait VfsFile: Sized {
	fn maybe_new<'a>(url: impl Into<UrlCow<'a>>) -> impl Future<Output = io::Result<Option<Self>>>;

	fn from_follow(url: UrlBuf, stat: Stat) -> impl Future<Output = Self>;
}

impl VfsFile for File {
	async fn maybe_new<'a>(url: impl Into<UrlCow<'a>>) -> io::Result<Option<Self>> {
		let url = url.into();
		ok_or_not_found(engine::file(&url).await.map(Some))
	}

	async fn from_follow(url: UrlBuf, lstat: Stat) -> Self {
		let link_to = if lstat.is_link() { engine::read_link(&url).await.ok() } else { None };
		let stat = Stat::from_follow(&url, lstat).await;

		Self { url, stat, extra: FileExtra::new(lstat, link_to, None) }
	}
}
