use std::{ffi::OsStr, hash::{BuildHasher, Hash, Hasher}, ops::Deref, path::{Path, PathBuf}};

use anyhow::Result;
use yazi_shared::url::{Uri, UrlBuf, UrlCow, Urn};

use crate::{cha::{Cha, ChaType}, provider};

#[derive(Clone, Debug, Default)]
pub struct File {
	pub url:     UrlBuf,
	pub cha:     Cha,
	pub link_to: Option<PathBuf>,
}

impl Deref for File {
	type Target = Cha;

	fn deref(&self) -> &Self::Target { &self.cha }
}

impl File {
	#[inline]
	pub async fn new(url: impl Into<UrlCow<'_>>) -> Result<Self> {
		let url = url.into();
		let cha = provider::symlink_metadata(&url).await?;
		Ok(Self::from_follow(url.into_owned(), cha).await)
	}

	#[inline]
	pub async fn from_follow(url: UrlBuf, cha: Cha) -> Self {
		let link_to = if cha.is_link() { provider::read_link(&url).await.ok() } else { None };

		let cha = Cha::from_follow(&url, cha).await;

		Self { url, cha, link_to }
	}

	#[inline]
	pub fn from_dummy(url: impl Into<UrlBuf>, r#type: Option<ChaType>) -> Self {
		let url = url.into();
		let cha = Cha::from_dummy(&url, r#type);
		Self { url, cha, link_to: None }
	}

	#[inline]
	pub fn hash_u64(&self) -> u64 { foldhash::fast::FixedState::default().hash_one(self) }

	#[inline]
	pub fn chdir(&self, wd: &Path) -> Self {
		Self { url: self.url.rebase(wd), cha: self.cha, link_to: self.link_to.clone() }
	}
}

impl File {
	// --- Url
	#[inline]
	pub fn url_owned(&self) -> UrlBuf { self.url.to_owned() }

	#[inline]
	pub fn uri(&self) -> &Uri { self.url.uri() }

	#[inline]
	pub fn urn(&self) -> &Urn { self.url.urn() }

	#[inline]
	pub fn name(&self) -> Option<&OsStr> { self.url.name() }

	#[inline]
	pub fn stem(&self) -> Option<&OsStr> { self.url.stem() }
}

impl Hash for File {
	fn hash<H: Hasher>(&self, state: &mut H) {
		self.url.hash(state);
		self.cha.len.hash(state);
		self.cha.btime.hash(state);
		self.cha.mtime.hash(state);
	}
}
