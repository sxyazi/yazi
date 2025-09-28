use std::io;

use yazi_fs::cha::{Cha, ChaKind};
use yazi_shared::url::Url;

use crate::provider;

pub trait VfsCha: Sized {
	fn from_url<'a>(url: impl Into<Url<'a>>) -> impl Future<Output = io::Result<Self>>;

	fn from_follow<'a, U>(url: U, cha: Self) -> impl Future<Output = Self>
	where
		U: Into<Url<'a>>;
}

impl VfsCha for Cha {
	#[inline]
	async fn from_url<'a>(url: impl Into<Url<'a>>) -> io::Result<Self> {
		let url = url.into();
		Ok(Self::from_follow(url, provider::symlink_metadata(url).await?).await)
	}

	async fn from_follow<'a, U>(url: U, mut cha: Self) -> Self
	where
		U: Into<Url<'a>>,
	{
		let url: Url = url.into();
		let mut retain = cha.kind & (ChaKind::HIDDEN | ChaKind::SYSTEM);

		if cha.is_link() {
			retain |= ChaKind::FOLLOW;
			cha = provider::metadata(url).await.unwrap_or(cha);
		}

		cha.attach(retain)
	}
}
