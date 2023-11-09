use std::{borrow::Cow, collections::BTreeMap, ffi::OsStr, fs::Metadata, ops::Deref};

use anyhow::Result;
use tokio::fs;
use yazi_shared::{Cha, ChaMeta, Url};

#[derive(Clone, Debug, Default)]
pub struct File {
	pub url:            Url,
	pub cha:            Cha,
	pub(super) link_to: Option<Url>,
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
		let mut cm = ChaMeta::empty();

		let (is_link, mut link_to) = (meta.is_symlink(), None);
		if is_link {
			cm |= ChaMeta::LINK;
			meta = fs::metadata(&url).await.unwrap_or(meta);
			link_to = fs::read_link(&url).await.map(Url::from).ok();
		}

		if is_link && meta.is_symlink() {
			cm |= ChaMeta::BAD_LINK;
		}

		if url.is_hidden() {
			cm |= ChaMeta::HIDDEN;
		}

		Self { url, cha: Cha::from(meta).with_meta(cm), link_to }
	}

	#[inline]
	pub fn from_dummy(url: Url) -> Self {
		let cm = if url.is_hidden() { ChaMeta::HIDDEN } else { ChaMeta::empty() };
		Self { url, cha: Cha::default().with_meta(cm), link_to: None }
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
	pub fn name_display(&self) -> Option<Cow<str>> {
		self.url.file_name().map(|s| s.to_string_lossy())
	}

	#[inline]
	pub fn stem(&self) -> Option<&OsStr> { self.url.file_stem() }

	#[inline]
	pub fn parent(&self) -> Option<Url> { self.url.parent_url() }

	// --- Link to / Is link
	#[inline]
	pub fn link_to(&self) -> Option<&Url> { self.link_to.as_ref() }
}
