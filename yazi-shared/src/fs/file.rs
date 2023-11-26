use std::{collections::BTreeMap, ffi::OsStr, fs::Metadata, ops::Deref};

use anyhow::Result;
use tokio::fs;

use crate::fs::{Cha, ChaKind, Url};

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
	pub async fn from(url: Url) -> Result<Self> {
		let meta = fs::symlink_metadata(&url).await?;
		Ok(Self::from_meta(url, meta).await)
	}

	pub async fn from_meta(url: Url, mut meta: Metadata) -> Self {
		let mut ck = ChaKind::empty();
		let (is_link, mut link_to) = (meta.is_symlink(), None);

		if is_link {
			ck |= ChaKind::LINK;
			meta = fs::metadata(&url).await.unwrap_or(meta);
			link_to = fs::read_link(&url).await.map(Url::from).ok();
		}

		if is_link && meta.is_symlink() {
			ck |= ChaKind::BAD_LINK;
		}

		if url.is_hidden() {
			ck |= ChaKind::HIDDEN;
		}

		Self { url, cha: Cha::from(meta).with_kind(ck), link_to }
	}

	#[inline]
	pub fn from_dummy(url: Url) -> Self {
		let ck = if url.is_hidden() { ChaKind::HIDDEN } else { ChaKind::empty() };
		Self { url, cha: Cha::default().with_kind(ck), link_to: None }
	}

	#[inline]
	pub fn into_map(self) -> BTreeMap<Url, File> {
		let mut map = BTreeMap::new();
		map.insert(self.url.clone(), self);
		map
	}
}

impl File {
	// --- Url
	#[inline]
	pub fn url(&self) -> Url { self.url.clone() }

	#[inline]
	pub fn name(&self) -> Option<&OsStr> { self.url.file_name() }

	#[inline]
	pub fn stem(&self) -> Option<&OsStr> { self.url.file_stem() }

	#[inline]
	pub fn parent(&self) -> Option<Url> { self.url.parent_url() }
}
