use std::io;

use yazi_fs::{cha::Cha, file::{File, FileExtra}};
use yazi_shared::url::{UrlBuf, UrlCow};

use crate::{VfsCha, engine};

pub trait VfsFile: Sized {
	fn maybe_new<'a>(url: impl Into<UrlCow<'a>>) -> impl Future<Output = io::Result<Option<Self>>>;

	fn from_follow(url: UrlBuf, cha: Cha) -> impl Future<Output = Self>;
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

	async fn from_follow(url: UrlBuf, cha: Cha) -> Self {
		let link_to = if cha.is_link() { engine::read_link(&url).await.ok() } else { None };
		let cha = Cha::from_follow(&url, cha).await;

		Self { url, cha, extra: FileExtra::new(link_to, None) }
	}
}
