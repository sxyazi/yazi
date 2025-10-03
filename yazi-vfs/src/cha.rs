use std::io;

use yazi_fs::cha::{Cha, ChaKind};
use yazi_shared::url::AsUrl;

use crate::provider;

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
		Ok(Self::from_follow(url, provider::symlink_metadata(url).await?).await)
	}

	async fn from_follow<U>(url: U, mut cha: Self) -> Self
	where
		U: AsUrl,
	{
		let url = url.as_url();
		let mut retain = cha.kind & (ChaKind::HIDDEN | ChaKind::SYSTEM);

		if cha.is_link() {
			retain |= ChaKind::FOLLOW;
			cha = provider::metadata(url).await.unwrap_or(cha);
		}

		cha.attach(retain)
	}
}
