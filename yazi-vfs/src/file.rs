use anyhow::Result;
use yazi_fs::{File, cha::Cha};
use yazi_shared::url::{UrlBuf, UrlCow};

use crate::{VfsCha, provider};

pub trait VfsFile: Sized {
	fn new<'a>(url: impl Into<UrlCow<'a>>) -> impl Future<Output = Result<Self>>;

	fn from_follow(url: UrlBuf, cha: Cha) -> impl Future<Output = Self>;
}

impl VfsFile for File {
	#[inline]
	async fn new<'a>(url: impl Into<UrlCow<'a>>) -> Result<Self> {
		let url = url.into();
		let cha = provider::symlink_metadata(&url).await?;
		Ok(Self::from_follow(url.into_owned(), cha).await)
	}

	#[inline]
	async fn from_follow(url: UrlBuf, cha: Cha) -> Self {
		let link_to = if cha.is_link() { provider::read_link(&url).await.ok() } else { None };

		let cha = Cha::from_follow(&url, cha).await;

		Self { url, cha, link_to }
	}
}
