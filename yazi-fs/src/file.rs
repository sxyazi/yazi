use std::{ffi::OsStr, fs::{FileType, Metadata}, hash::{BuildHasher, Hash, Hasher}, ops::Deref};

use anyhow::Result;
use tokio::fs;
use yazi_shared::url::{Url, Urn, UrnBuf};

use crate::cha::Cha;

#[derive(Clone, Debug, Default)]
pub struct File {
	pub url:     Url,
	pub cha:     Cha,
	pub link_to: Option<Url>,
}

impl Deref for File {
	type Target = Cha;

	#[inline]
	fn deref(&self) -> &Self::Target { &self.cha }
}

impl File {
	#[inline]
	pub async fn new(url: Url) -> Result<Self> {
		let meta = fs::symlink_metadata(&url).await?;
		Ok(Self::from_follow(url, meta).await)
	}

	#[inline]
	pub async fn from_follow(url: Url, meta: Metadata) -> Self {
		let link_to =
			if meta.is_symlink() { fs::read_link(&url).await.map(Url::from).ok() } else { None };

		let cha = Cha::from_follow(&url, meta).await;

		Self { url, cha, link_to }
	}

	#[inline]
	pub fn from_dummy(url: Url, ft: Option<FileType>) -> Self {
		let cha = Cha::from_dummy(&url, ft);
		Self { url, cha, link_to: None }
	}

	#[inline]
	pub fn hash_u64(&self) -> u64 { foldhash::fast::FixedState::default().hash_one(self) }

	#[inline]
	pub fn rebase(&self, parent: &Url) -> Self {
		Self { url: self.url.rebase(parent), cha: self.cha, link_to: self.link_to.clone() }
	}
}

impl File {
	// --- Url
	#[inline]
	pub fn url_owned(&self) -> Url { self.url.to_owned() }

	#[inline]
	pub fn urn(&self) -> &Urn { self.url.urn() }

	#[inline]
	pub fn urn_owned(&self) -> UrnBuf { self.url.urn_owned() }

	#[inline]
	pub fn name(&self) -> &OsStr { self.url.name() }

	#[inline]
	pub fn stem(&self) -> Option<&OsStr> { self.url.file_stem() }
}

impl Hash for File {
	fn hash<H: Hasher>(&self, state: &mut H) {
		self.url.hash(state);
		self.cha.len.hash(state);
		self.cha.btime.hash(state);
		self.cha.mtime.hash(state);
	}
}
