use std::{cell::Cell, ffi::OsStr, fs::{FileType, Metadata}, ops::Deref};

use anyhow::Result;
use tokio::fs;

use super::{Urn, UrnBuf};
use crate::{fs::{Cha, ChaKind, Url}, theme::IconCache};

#[derive(Clone, Debug, Default)]
pub struct File {
	pub url:     Url,
	pub cha:     Cha,
	pub link_to: Option<Url>,
	pub icon:    Cell<IconCache>,
}

impl Deref for File {
	type Target = Cha;

	#[inline]
	fn deref(&self) -> &Self::Target { &self.cha }
}

impl AsRef<File> for File {
	#[inline]
	fn as_ref(&self) -> &File { self }
}

impl File {
	#[inline]
	pub async fn from(url: Url) -> Result<Self> {
		let meta = fs::symlink_metadata(&url).await?;
		Ok(Self::from_meta(url, meta).await)
	}

	#[inline]
	pub async fn from_meta(url: Url, mut meta: Metadata) -> Self {
		let mut ck = ChaKind::empty();
		let (is_link, mut link_to) = (meta.is_symlink(), None);

		if is_link {
			meta = fs::metadata(&url).await.unwrap_or(meta);
			link_to = fs::read_link(&url).await.map(Url::from).ok();
		}

		if is_link && meta.is_symlink() {
			ck |= ChaKind::ORPHAN;
		} else if is_link {
			ck |= ChaKind::LINK;
		}

		#[cfg(unix)]
		if url.is_hidden() {
			ck |= ChaKind::HIDDEN;
		}
		#[cfg(windows)]
		{
			use std::os::windows::fs::MetadataExt;
			if meta.file_attributes() & 2 != 0 {
				ck |= ChaKind::HIDDEN;
			}
		}

		Self { url, cha: Cha::from(meta).with_kind(ck), link_to, icon: Default::default() }
	}

	#[inline]
	pub fn from_dummy(url: Url, ft: Option<FileType>) -> Self {
		Self {
			url,
			cha: ft.map_or_else(Cha::dummy, Cha::from),
			link_to: None,
			icon: Default::default(),
		}
	}

	#[inline]
	pub fn rebase(&self, parent: &Url) -> Self {
		Self {
			url:     self.url.rebase(parent),
			cha:     self.cha,
			link_to: self.link_to.clone(),
			icon:    Default::default(),
		}
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

	#[inline]
	pub fn parent(&self) -> Option<Url> { self.url.parent_url() }
}
