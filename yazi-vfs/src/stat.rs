use std::io;

use yazi_fs::stat::Stat;
use yazi_shared::url::AsUrl;

use crate::engine;

pub trait VfsStat: Sized {
	fn from_url(url: impl AsUrl) -> impl Future<Output = io::Result<Self>>;

	fn from_follow<U>(url: U, stat: Self) -> impl Future<Output = Self>
	where
		U: AsUrl;
}

impl VfsStat for Stat {
	#[inline]
	async fn from_url(url: impl AsUrl) -> io::Result<Self> {
		let url = url.as_url();
		Ok(Self::from_follow(url, engine::symlink_metadata(url).await?).await)
	}

	async fn from_follow<U>(url: U, stat: Self) -> Self
	where
		U: AsUrl,
	{
		let url = url.as_url();
		let followed = if stat.is_link() { engine::metadata(url).await.ok() } else { None };
		stat.follow(followed)
	}
}
