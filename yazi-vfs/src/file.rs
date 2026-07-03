use std::io;

use yazi_fs::{cha::Cha, file::File};
use yazi_shared::url::{UrlBuf, UrlCow};

use crate::{VfsCha, provider};

pub trait VfsFile: Sized {
	fn new<'a>(url: impl Into<UrlCow<'a>>) -> impl Future<Output = io::Result<Self>>;

	fn maybe_new<'a>(url: impl Into<UrlCow<'a>>) -> impl Future<Output = io::Result<Option<Self>>>;

	fn from_follow(url: UrlBuf, cha: Cha) -> impl Future<Output = Self>;
}

impl VfsFile for File {
	async fn new<'a>(url: impl Into<UrlCow<'a>>) -> io::Result<Self> {
		let url = url.into();
		let cha = provider::symlink_metadata(&url).await?;
		Ok(Self::from_follow(url.into_owned(), cha).await)
	}

	async fn maybe_new<'a>(url: impl Into<UrlCow<'a>>) -> io::Result<Option<Self>> {
		let url = url.into();
		let cha = match provider::symlink_metadata(&url).await {
			Ok(cha) => cha,
			Err(e) if e.kind() == io::ErrorKind::NotFound => return Ok(None),
			Err(e) => return Err(e),
		};
		Ok(Some(Self::from_follow(url.into_owned(), cha).await))
	}

	async fn from_follow(url: UrlBuf, cha: Cha) -> Self {
		let link_to = if cha.is_link() { provider::read_link(&url).await.ok() } else { None };

		let cha = Cha::from_follow(&url, cha).await;

		Self { url, cha, link_to }
	}
}
