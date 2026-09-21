use std::{borrow::Cow, hash::{Hash, Hasher}, ops::Deref, path::{Path, PathBuf}};

use hashbrown::Equivalent;
use serde::{Deserialize, Serialize};
use yazi_shared::{path::PathDyn, strand::Strand, url::{AsUrl, Url, UrlBuf, UrlLike}};

use crate::{FsUrl, file::{FileExtra, FileExtraInner}, stat::{Stat, StatType}};

#[derive(Clone, Debug, Default, Deserialize)]
pub struct File {
	pub url:   UrlBuf,
	pub stat:  Stat,
	#[serde(flatten)]
	pub extra: FileExtra,
}

impl Deref for File {
	type Target = Stat;

	fn deref(&self) -> &Self::Target { &self.stat }
}

impl PartialEq for File {
	fn eq(&self, other: &Self) -> bool { self.url == other.url }
}

impl Eq for File {}

impl Hash for File {
	fn hash<H: Hasher>(&self, state: &mut H) { self.url.hash(state); }
}

impl From<&Self> for File {
	fn from(value: &Self) -> Self { value.clone() }
}

impl From<File> for Cow<'_, File> {
	fn from(value: File) -> Self { Cow::Owned(value) }
}

impl<'a> From<&'a File> for Cow<'a, File> {
	fn from(value: &'a File) -> Self { Cow::Borrowed(value) }
}

impl AsUrl for File {
	fn as_url(&self) -> Url<'_> { self.url.as_url() }
}

impl AsUrl for &File {
	fn as_url(&self) -> Url<'_> { self.url.as_url() }
}

impl UrlLike for File {}

impl Equivalent<File> for Url<'_> {
	fn equivalent(&self, key: &File) -> bool { *self == key.url }
}

impl Equivalent<File> for UrlBuf {
	fn equivalent(&self, key: &File) -> bool { self == key.url }
}

impl File {
	#[inline]
	pub fn hits(&self, other: &Self) -> bool {
		self.stat.hits(other.stat) && self.lstat().hits(other.lstat())
	}

	#[inline]
	pub fn lstat(&self) -> Stat { self.extra.lstat().unwrap_or(self.stat) }

	#[inline]
	pub fn cache(&self) -> Option<PathBuf> {
		if self.is_dir() { self.url.cache_bucket() } else { self.url.cache_entry() }
	}

	#[inline]
	pub fn from_dummy(url: impl Into<UrlBuf>, r#type: Option<StatType>) -> Self {
		let url = url.into();
		let stat = Stat::from_dummy(&url, r#type);
		Self { url, stat, extra: Default::default() }
	}

	#[inline]
	pub(crate) fn chdir(&self, wd: &Path) -> Self {
		Self { url: self.url.rebase(wd), stat: self.stat, extra: self.extra.clone() }
	}

	#[inline]
	pub fn content_path(&self) -> Cow<'_, Path> {
		if let Some(backing) = self.extra.backing() {
			backing.into()
		} else if let Some(local) = self.as_local() {
			local.into()
		} else {
			self.cache().expect("non-local URL should have a cache path").into()
		}
	}
}

impl File {
	// --- Url
	#[inline]
	pub fn urn(&self) -> PathDyn<'_> { self.url.urn() }

	#[inline]
	pub fn key(&self) -> PathDyn<'_> { self.url.key() }

	#[inline]
	pub fn name(&self) -> Option<Strand<'_>> { self.url.name() }

	#[inline]
	pub fn stem(&self) -> Option<Strand<'_>> { self.url.stem() }
}

impl Serialize for File {
	fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
		#[derive(Serialize)]
		struct Shadow<'a> {
			url:   &'a UrlBuf,
			stat:  Stat,
			#[serde(flatten)]
			extra: &'a FileExtraInner,
		}

		let fallback = FileExtraInner { lstat: self.stat, ..Default::default() };
		Shadow { url: &self.url, stat: self.stat, extra: self.extra.as_ref().unwrap_or(&fallback) }
			.serialize(serializer)
	}
}
