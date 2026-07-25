use std::io;

use yazi_fs::cha::Cha;
use yazi_shared::url::AsUrl;

use crate::engine;

pub trait VfsCha: Sized {
	fn from_url(url: impl AsUrl) -> impl Future<Output = io::Result<Self>>;

	fn from_follow<U>(url: U, cha: Self) -> impl Future<Output = Self>
	where
		U: AsUrl;
}

impl VfsCha for Cha {
	#[inline]
	async fn from_url(url: impl AsUrl) -> io::Result<Self> {
		let url = url.as_url();
		Ok(Self::from_follow(url, engine::symlink_metadata(url).await?).await)
	}

	async fn from_follow<U>(url: U, cha: Self) -> Self
	where
		U: AsUrl,
	{
		let url = url.as_url();
		let followed = if cha.is_link() { engine::metadata(url).await.ok() } else { None };
		cha.follow(followed)
	}
}
