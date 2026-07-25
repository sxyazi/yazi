use std::{borrow::Cow, ops::Deref, path::{Path, PathBuf}};

use serde::{Deserialize, Serialize};
use yazi_shared::{path::PathDyn, strand::Strand, url::{AsUrl, Url, UrlBuf, UrlLike}};

use crate::{FsUrl, cha::{Cha, ChaType}, file::FileExtra};

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct File {
	pub url:   UrlBuf,
	pub cha:   Cha,
	#[serde(flatten)]
	pub extra: FileExtra,
}

impl Deref for File {
	type Target = Cha;

	fn deref(&self) -> &Self::Target { &self.cha }
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

impl File {
	#[inline]
	pub fn cache(&self) -> Option<PathBuf> {
		if self.is_dir() { self.url.cache_bucket() } else { self.url.cache_entry() }
	}

	#[inline]
	pub fn from_dummy(url: impl Into<UrlBuf>, r#type: Option<ChaType>) -> Self {
		let url = url.into();
		let cha = Cha::from_dummy(&url, r#type);
		Self { url, cha, extra: Default::default() }
	}

	#[inline]
	pub fn chdir(&self, wd: &Path) -> Self {
		Self { url: self.url.rebase(wd), cha: self.cha, extra: self.extra.clone() }
	}

	#[inline]
	pub fn content_path(&self) -> Cow<'_, Path> {
		if let Some(backing) = self.extra.backing() {
			backing.into()
		} else if let Some(local) = self.url.as_local() {
			local.into()
		} else {
			self.cache().expect("non-local URL should have a cache path").into()
		}
	}
}

impl File {
	// --- Url
	#[inline]
	pub fn url_owned(&self) -> UrlBuf { self.url.clone() }

	#[inline]
	pub fn uri(&self) -> PathDyn<'_> { self.url.uri() }

	#[inline]
	pub fn urn(&self) -> PathDyn<'_> { self.url.urn() }

	#[inline]
	pub fn entry_key(&self) -> PathDyn<'_> { self.url.entry_key() }

	#[inline]
	pub fn name(&self) -> Option<Strand<'_>> { self.url.name() }

	#[inline]
	pub fn stem(&self) -> Option<Strand<'_>> { self.url.stem() }
}
